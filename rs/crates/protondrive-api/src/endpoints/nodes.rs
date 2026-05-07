use crate::client::ApiClient;
use protondrive_core::{error::Result, ids::{NodeId, ShareId}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LinkListResponse {
    #[serde(rename = "Links")]
    pub links: Vec<LinkDto>,
    #[serde(rename = "Total")]
    pub total: u64,
}

#[derive(Debug, Deserialize)]
pub struct LinkDto {
    #[serde(rename = "LinkID")]
    pub link_id: String,
    #[serde(rename = "ParentLinkID")]
    pub parent_link_id: Option<String>,
    #[serde(rename = "Type")]
    pub r#type: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "State")]
    pub state: u32,
    #[serde(rename = "CreateTime")]
    pub create_time: u64,
    #[serde(rename = "ModifyTime")]
    pub modify_time: u64,
    #[serde(rename = "MIMEType")]
    pub mime_type: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<u64>,
    #[serde(rename = "NodeKey")]
    pub node_key: Option<String>,
    #[serde(rename = "NodePassphrase")]
    pub node_passphrase: Option<String>,
    #[serde(rename = "NodePassphraseSignature")]
    pub node_passphrase_signature: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RenameRequest {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "MIMEType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MoveRequest {
    #[serde(rename = "ParentLinkID")]
    pub parent_link_id: String,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TrashRequest {
    #[serde(rename = "LinkIDs")]
    pub link_ids: Vec<String>,
}

impl ApiClient {
    pub async fn list_children(
        &self,
        share_id: &ShareId,
        parent_id: Option<&NodeId>,
    ) -> Result<LinkListResponse> {
        let path = match parent_id {
            Some(pid) => format!("/drive/shares/{share_id}/folders/{pid}/children"),
            None => format!("/drive/shares/{share_id}/links"),
        };
        self.get(&path).await
    }

    pub async fn get_link(&self, share_id: &ShareId, link_id: &NodeId) -> Result<LinkDto> {
        #[derive(Deserialize)]
        struct Wrap {
            #[serde(rename = "Link")]
            link: LinkDto,
        }
        let r: Wrap = self
            .get(&format!("/drive/shares/{share_id}/links/{link_id}"))
            .await?;
        Ok(r.link)
    }

    pub async fn rename_link(
        &self,
        share_id: &ShareId,
        link_id: &NodeId,
        req: &RenameRequest,
    ) -> Result<()> {
        self.put::<_, serde_json::Value>(
            &format!("/drive/shares/{share_id}/links/{link_id}"),
            req,
        )
        .await?;
        Ok(())
    }

    pub async fn move_link(
        &self,
        share_id: &ShareId,
        link_id: &NodeId,
        req: &MoveRequest,
    ) -> Result<()> {
        self.put::<_, serde_json::Value>(
            &format!("/drive/shares/{share_id}/links/{link_id}/move"),
            req,
        )
        .await?;
        Ok(())
    }

    pub async fn trash_links(&self, share_id: &ShareId, req: &TrashRequest) -> Result<()> {
        self.post::<_, serde_json::Value>(
            &format!("/drive/shares/{share_id}/trash"),
            req,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_link(&self, share_id: &ShareId, link_id: &NodeId) -> Result<()> {
        self.delete(&format!("/drive/shares/{share_id}/links/{link_id}"))
            .await
    }
}
