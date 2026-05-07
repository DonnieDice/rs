use crate::client::ApiClient;
use protondrive_core::{error::Result, ids::ShareId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ShareListResponse {
    #[serde(rename = "Shares")]
    pub shares: Vec<ShareDto>,
}

#[derive(Debug, Deserialize)]
pub struct ShareDto {
    #[serde(rename = "ShareID")]
    pub share_id: String,
    #[serde(rename = "VolumeID")]
    pub volume_id: String,
    #[serde(rename = "LinkID")]
    pub link_id: String,
    #[serde(rename = "Flags")]
    pub flags: u32,
}

#[derive(Debug, Serialize)]
pub struct CreateShareRequest {
    #[serde(rename = "AddressID")]
    pub address_id: String,
    #[serde(rename = "Name")]
    pub name: String,
}

impl ApiClient {
    pub async fn list_shares(&self) -> Result<ShareListResponse> {
        self.get("/drive/shares").await
    }

    pub async fn create_share(&self, req: &CreateShareRequest) -> Result<ShareDto> {
        #[derive(Deserialize)]
        struct Wrap {
            #[serde(rename = "Share")]
            share: ShareDto,
        }
        let r: Wrap = self.post("/drive/shares", req).await?;
        Ok(r.share)
    }
}
