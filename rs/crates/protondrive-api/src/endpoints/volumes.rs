use crate::client::ApiClient;
use protondrive_core::{error::Result, ids::VolumeId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VolumeListResponse {
    #[serde(rename = "Volumes")]
    pub volumes: Vec<VolumeDto>,
}

#[derive(Debug, Deserialize)]
pub struct VolumeDto {
    #[serde(rename = "VolumeID")]
    pub volume_id: String,
    #[serde(rename = "Share")]
    pub share: ShareRef,
    #[serde(rename = "State")]
    pub state: u32,
}

#[derive(Debug, Deserialize)]
pub struct ShareRef {
    #[serde(rename = "ShareID")]
    pub share_id: String,
    #[serde(rename = "LinkID")]
    pub link_id: String,
}

impl ApiClient {
    pub async fn list_volumes(&self) -> Result<VolumeListResponse> {
        self.get("/drive/volumes").await
    }
}
