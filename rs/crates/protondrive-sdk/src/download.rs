use bytes::Bytes;
use futures::Stream;
use protondrive_api::client::ApiClient;
use protondrive_core::{error::Result, ids::NodeId};
use std::pin::Pin;

pub type DownloadStream = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;

pub async fn download(_client: &ApiClient, _node: &NodeId) -> Result<DownloadStream> {
    // Phase 1 deliverable. Stub for API surface completeness.
    // TODO Phase 1: fetch block URLs, stream decrypt via protondrive-crypto
    Err(protondrive_core::error::DriveError::Auth(
        "download not yet implemented (Phase 1)".into(),
    ))
}
