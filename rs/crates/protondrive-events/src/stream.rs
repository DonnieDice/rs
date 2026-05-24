use crate::types::{DriveEvent, EventAction};
use futures::Stream;
use protondrive_api::client::ApiClient;
use protondrive_core::{
    error::{DriveError, Result},
    ids::{EventId, ShareId},
};
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::{debug, warn};

/// A lazy, poll-driven stream of Drive events for a single share.
///
/// Calls `/drive/shares/{share_id}/events/{since}` in a loop.
/// Yields `DriveEvent` items; terminates on unrecoverable errors.
pub struct EventStream {
    client: ApiClient,
    share_id: ShareId,
    cursor: EventId,
    // Buffer of events not yet yielded
    buffer: Vec<DriveEvent>,
    // Whether we have more pages to fetch before sleeping
    has_more: bool,
    // Pending async fetch
    fetch: Option<Pin<Box<dyn std::future::Future<Output = Result<FetchResult>> + Send>>>,
}

struct FetchResult {
    events: Vec<DriveEvent>,
    next_cursor: EventId,
    more: bool,
}

impl EventStream {
    pub fn new(client: ApiClient, share_id: ShareId, since: EventId) -> Self {
        Self {
            client,
            share_id,
            cursor: since,
            buffer: Vec::new(),
            has_more: true,
            fetch: None,
        }
    }
}

impl Stream for EventStream {
    type Item = Result<DriveEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Yield buffered events first
        if let Some(event) = self.buffer.pop() {
            return Poll::Ready(Some(Ok(event)));
        }

        // Start a fetch if none is in flight
        if self.fetch.is_none() {
            let client = self.client.clone();
            let share_id = self.share_id.clone();
            let cursor = self.cursor.clone();

            self.fetch = Some(Box::pin(async move {
                let resp = client.get_events(&share_id, &cursor).await?;
                let next_cursor = EventId::new(&resp.event_id);
                let more = resp.more != 0;
                let events: Vec<DriveEvent> = resp
                    .events
                    .into_iter()
                    .map(|e| DriveEvent {
                        event_id: EventId::new(&e.event_id),
                        share_id: share_id.clone(),
                        action: EventAction::from_api(e.action, e.link),
                    })
                    .collect();
                Ok(FetchResult {
                    events,
                    next_cursor,
                    more,
                })
            }));
        }

        // Poll the in-flight fetch
        if let Some(fut) = self.fetch.as_mut() {
            match fut.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(result) => {
                    self.fetch = None;
                    match result {
                        Err(DriveError::EventStreamGap { latest_event_id }) => {
                            warn!("event stream gap detected; consumer must re-sync");
                            return Poll::Ready(Some(Err(DriveError::EventStreamGap {
                                latest_event_id,
                            })));
                        }
                        Err(e) => return Poll::Ready(Some(Err(e))),
                        Ok(r) => {
                            self.cursor = r.next_cursor;
                            self.has_more = r.more;
                            self.buffer = r.events;
                            self.buffer.reverse(); // pop() from end = FIFO
                            debug!(
                                count = self.buffer.len(),
                                more = self.has_more,
                                "fetched events"
                            );
                        }
                    }
                }
            }
        }

        // Yield first buffered event or signal "no items right now"
        match self.buffer.pop() {
            Some(event) => Poll::Ready(Some(Ok(event))),
            None => {
                // No events yet; re-register wakeup and return Pending
                // (The caller should sleep and re-poll)
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
