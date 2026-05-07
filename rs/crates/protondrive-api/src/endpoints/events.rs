use crate::client::ApiClient;
use protondrive_core::{error::Result, ids::{EventId, ShareId}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EventsResponse {
    #[serde(rename = "EventID")]
    pub event_id: String,
    #[serde(rename = "More")]
    pub more: u8,
    #[serde(rename = "Events")]
    pub events: Vec<EventDto>,
}

#[derive(Debug, Deserialize)]
pub struct EventDto {
    #[serde(rename = "EventID")]
    pub event_id: String,
    #[serde(rename = "Action")]
    pub action: u32,
    #[serde(rename = "Link")]
    pub link: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct LatestEventResponse {
    #[serde(rename = "EventID")]
    pub event_id: String,
}

impl ApiClient {
    pub async fn get_latest_event_id(&self, share_id: &ShareId) -> Result<LatestEventResponse> {
        self.get(&format!("/drive/shares/{share_id}/events/latest"))
            .await
    }

    pub async fn get_events(
        &self,
        share_id: &ShareId,
        since: &EventId,
    ) -> Result<EventsResponse> {
        self.get(&format!("/drive/shares/{share_id}/events/{since}"))
            .await
    }
}
