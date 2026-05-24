use protondrive_api::client::ApiClient;
use protondrive_core::error::{DriveError, Result};
use protondrive_core::ids::NodeId;
use tokio::io::{AsyncRead, AsyncReadExt};

const BLOCK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

#[derive(Debug, Clone)]
pub struct UploadOptions {
    /// Number of blocks to upload in parallel. Default: 4.
    pub concurrency: usize,
    pub mime_type: Option<String>,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            concurrency: 4,
            mime_type: None,
        }
    }
}

pub async fn upload<R: AsyncRead + Unpin + Send>(
    _client: &ApiClient,
    _parent: &NodeId,
    _name: &str,
    mut content: R,
    _opts: UploadOptions,
) -> Result<NodeId> {
    // Phase 2 deliverable. Reads and discards the stream to validate the
    // API signature compiles correctly end-to-end.
    let mut buf = vec![0u8; BLOCK_SIZE];
    loop {
        let n = content
            .read(&mut buf)
            .await
            .map_err(|e| DriveError::Network(e.to_string()))?;
        if n == 0 {
            break;
        }
    }

    // TODO Phase 2: create revision, encrypt blocks, upload blocks, commit revision
    Err(DriveError::Other(anyhow::anyhow!(
        "upload not yet implemented (Phase 2)"
    )))
}
