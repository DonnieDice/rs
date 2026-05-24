use crate::client::ApiClient;
use protondrive_core::{
    error::Result,
    ids::{NodeId, RevisionId, ShareId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RevisionListResponse {
    #[serde(rename = "Revisions")]
    pub revisions: Vec<RevisionDto>,
}

#[derive(Debug, Deserialize)]
pub struct RevisionDto {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "State")]
    pub state: u32,
    #[serde(rename = "CreateTime")]
    pub create_time: u64,
    #[serde(rename = "ManifestSignature")]
    pub manifest_signature: String,
}

#[derive(Debug, Serialize)]
pub struct RestoreRevisionRequest {
    #[serde(rename = "RevisionID")]
    pub revision_id: String,
}

impl ApiClient {
    pub async fn list_revisions(
        &self,
        share_id: &ShareId,
        link_id: &NodeId,
    ) -> Result<RevisionListResponse> {
        self.get(&format!(
            "/drive/shares/{share_id}/files/{link_id}/revisions"
        ))
        .await
    }

    pub async fn restore_revision(
        &self,
        share_id: &ShareId,
        link_id: &NodeId,
        revision_id: &RevisionId,
    ) -> Result<()> {
        let req = RestoreRevisionRequest {
            revision_id: revision_id.to_string(),
        };
        self.put::<_, serde_json::Value>(
            &format!("/drive/shares/{share_id}/files/{link_id}/restore"),
            &req,
        )
        .await?;
        Ok(())
    }
}
