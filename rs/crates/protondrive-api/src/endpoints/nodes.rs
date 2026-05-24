use crate::client::ApiClient;
use protondrive_core::{
    error::Result,
    ids::{NodeId, ShareId, VolumeId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LinkListResponse {
    #[serde(rename = "Links")]
    pub links: Vec<LinkDto>,
    #[serde(rename = "Total")]
    pub total: u64,
}

#[derive(Debug, Deserialize)]
pub struct ChildrenResponse {
    #[serde(rename = "LinkIDs")]
    pub link_ids: Vec<String>,
    #[serde(rename = "More")]
    pub more: bool,
    #[serde(rename = "AnchorID")]
    pub anchor_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChildrenQuery<'a> {
    #[serde(rename = "FoldersOnly", skip_serializing_if = "Option::is_none")]
    pub folders_only: Option<u8>,
    #[serde(rename = "AnchorID", skip_serializing_if = "Option::is_none")]
    pub anchor_id: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct LinksMetadataRequest {
    #[serde(rename = "LinkIDs")]
    pub link_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LinksMetadataResponse {
    #[serde(rename = "Links")]
    pub links: Vec<serde_json::Value>,
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
    #[serde(rename = "NameSignatureEmail", skip_serializing_if = "Option::is_none")]
    pub name_signature_email: Option<String>,
    #[serde(rename = "Hash", skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(rename = "OriginalHash", skip_serializing_if = "Option::is_none")]
    pub original_hash: Option<String>,
    #[serde(rename = "MIMEType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MoveRequest {
    #[serde(rename = "ParentLinkID")]
    pub parent_link_id: String,
    #[serde(rename = "NodePassphrase", skip_serializing_if = "Option::is_none")]
    pub node_passphrase: Option<String>,
    #[serde(
        rename = "NodePassphraseSignature",
        skip_serializing_if = "Option::is_none"
    )]
    pub node_passphrase_signature: Option<String>,
    #[serde(rename = "SignatureEmail", skip_serializing_if = "Option::is_none")]
    pub signature_email: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "NameSignatureEmail", skip_serializing_if = "Option::is_none")]
    pub name_signature_email: Option<String>,
    #[serde(rename = "Hash", skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(rename = "OriginalHash", skip_serializing_if = "Option::is_none")]
    pub original_hash: Option<String>,
    #[serde(rename = "ContentHash", skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrashRequest {
    #[serde(rename = "LinkIDs")]
    pub link_ids: Vec<String>,
}

pub type RestoreRequest = TrashRequest;
pub type DeleteMultipleRequest = TrashRequest;
pub type RemoveMineRequest = TrashRequest;

#[derive(Debug, Deserialize)]
pub struct BatchLinkResponse {
    #[serde(rename = "LinkID")]
    pub link_id: String,
    #[serde(rename = "Response")]
    pub response: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct BatchLinkResponses {
    #[serde(rename = "Responses")]
    pub responses: Vec<BatchLinkResponse>,
}

#[derive(Debug, Deserialize)]
pub struct TrashPageResponse {
    #[serde(rename = "Trash")]
    pub trash: Vec<TrashShareLinks>,
}

#[derive(Debug, Deserialize)]
pub struct TrashShareLinks {
    #[serde(rename = "ShareID")]
    pub share_id: Option<String>,
    #[serde(rename = "LinkIDs")]
    pub link_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TrashPageQuery {
    #[serde(rename = "Page")]
    pub page: u32,
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
        self.put::<_, serde_json::Value>(&format!("/drive/shares/{share_id}/links/{link_id}"), req)
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
        self.post::<_, serde_json::Value>(&format!("/drive/shares/{share_id}/trash"), req)
            .await?;
        Ok(())
    }

    pub async fn delete_link(&self, share_id: &ShareId, link_id: &NodeId) -> Result<()> {
        self.delete(&format!("/drive/shares/{share_id}/links/{link_id}"))
            .await
    }

    pub async fn list_volume_children(
        &self,
        volume_id: &VolumeId,
        folder_id: &NodeId,
        query: &ChildrenQuery<'_>,
    ) -> Result<ChildrenResponse> {
        self.get(&volume_children_path(volume_id, folder_id, query)?)
            .await
    }

    pub async fn load_links_metadata(
        &self,
        volume_id: &VolumeId,
        req: &LinksMetadataRequest,
    ) -> Result<LinksMetadataResponse> {
        self.post(&volume_links_metadata_path(volume_id), req).await
    }

    pub async fn rename_volume_link(
        &self,
        volume_id: &VolumeId,
        link_id: &NodeId,
        req: &RenameRequest,
    ) -> Result<()> {
        self.put::<_, serde_json::Value>(&volume_link_rename_path(volume_id, link_id), req)
            .await?;
        Ok(())
    }

    pub async fn move_volume_link(
        &self,
        volume_id: &VolumeId,
        link_id: &NodeId,
        req: &MoveRequest,
    ) -> Result<()> {
        self.put::<_, serde_json::Value>(&volume_link_move_path(volume_id, link_id), req)
            .await?;
        Ok(())
    }

    pub async fn trash_volume_links(
        &self,
        volume_id: &VolumeId,
        req: &TrashRequest,
    ) -> Result<BatchLinkResponses> {
        self.post(&volume_trash_multiple_path(volume_id), req).await
    }

    pub async fn restore_volume_links(
        &self,
        volume_id: &VolumeId,
        req: &RestoreRequest,
    ) -> Result<BatchLinkResponses> {
        self.put(&volume_restore_multiple_path(volume_id), req)
            .await
    }

    pub async fn delete_trashed_volume_links(
        &self,
        volume_id: &VolumeId,
        req: &DeleteMultipleRequest,
    ) -> Result<BatchLinkResponses> {
        self.post(&volume_delete_trashed_multiple_path(volume_id), req)
            .await
    }

    pub async fn delete_my_volume_links(
        &self,
        volume_id: &VolumeId,
        req: &RemoveMineRequest,
    ) -> Result<BatchLinkResponses> {
        self.post(&volume_remove_mine_path(volume_id), req).await
    }

    pub async fn delete_volume_links(
        &self,
        volume_id: &VolumeId,
        req: &DeleteMultipleRequest,
    ) -> Result<BatchLinkResponses> {
        self.post(&volume_delete_multiple_path(volume_id), req)
            .await
    }

    pub async fn list_trashed_volume_links(
        &self,
        volume_id: &VolumeId,
        page: u32,
    ) -> Result<TrashPageResponse> {
        self.get(&volume_trash_path(volume_id, page)?).await
    }

    pub async fn empty_volume_trash(&self, volume_id: &VolumeId) -> Result<()> {
        self.delete(&format!("/drive/volumes/{volume_id}/trash"))
            .await
    }
}

pub(crate) fn volume_children_path(
    volume_id: &VolumeId,
    folder_id: &NodeId,
    query: &ChildrenQuery<'_>,
) -> Result<String> {
    let query = serde_urlencoded::to_string(query)
        .map_err(|e| protondrive_core::DriveError::Network(e.to_string()))?;
    let path = format!("/drive/v2/volumes/{volume_id}/folders/{folder_id}/children");
    Ok(if query.is_empty() {
        path
    } else {
        format!("{path}?{query}")
    })
}

pub(crate) fn volume_links_metadata_path(volume_id: &VolumeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/links")
}

pub(crate) fn volume_link_rename_path(volume_id: &VolumeId, link_id: &NodeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/links/{link_id}/rename")
}

pub(crate) fn volume_link_move_path(volume_id: &VolumeId, link_id: &NodeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/links/{link_id}/move")
}

pub(crate) fn volume_trash_multiple_path(volume_id: &VolumeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/trash_multiple")
}

pub(crate) fn volume_restore_multiple_path(volume_id: &VolumeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/trash/restore_multiple")
}

pub(crate) fn volume_delete_trashed_multiple_path(volume_id: &VolumeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/trash/delete_multiple")
}

pub(crate) fn volume_remove_mine_path(volume_id: &VolumeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/remove-mine")
}

pub(crate) fn volume_delete_multiple_path(volume_id: &VolumeId) -> String {
    format!("/drive/v2/volumes/{volume_id}/delete_multiple")
}

pub(crate) fn volume_trash_path(volume_id: &VolumeId, page: u32) -> Result<String> {
    let query = serde_urlencoded::to_string(TrashPageQuery { page })
        .map_err(|e| protondrive_core::DriveError::Network(e.to_string()))?;
    Ok(format!("/drive/volumes/{volume_id}/trash?{query}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_node_route_builders_match_current_drive_api() {
        let volume = VolumeId::from("volume");
        let link = NodeId::from("link");
        let query = ChildrenQuery {
            folders_only: Some(1),
            anchor_id: Some("anchor"),
        };

        assert_eq!(
            volume_children_path(&volume, &link, &query).unwrap(),
            "/drive/v2/volumes/volume/folders/link/children?FoldersOnly=1&AnchorID=anchor"
        );
        assert_eq!(
            volume_links_metadata_path(&volume),
            "/drive/v2/volumes/volume/links"
        );
        assert_eq!(
            volume_link_rename_path(&volume, &link),
            "/drive/v2/volumes/volume/links/link/rename"
        );
        assert_eq!(
            volume_link_move_path(&volume, &link),
            "/drive/v2/volumes/volume/links/link/move"
        );
        assert_eq!(
            volume_trash_multiple_path(&volume),
            "/drive/v2/volumes/volume/trash_multiple"
        );
        assert_eq!(
            volume_delete_trashed_multiple_path(&volume),
            "/drive/v2/volumes/volume/trash/delete_multiple"
        );
        assert_eq!(
            volume_remove_mine_path(&volume),
            "/drive/v2/volumes/volume/remove-mine"
        );
    }
}
