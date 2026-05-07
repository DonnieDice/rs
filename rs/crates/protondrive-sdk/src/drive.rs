use crate::download::DownloadStream;
use crate::upload::UploadOptions;
use protondrive_api::client::ApiClient;
use protondrive_auth::Session;
use protondrive_core::{
    error::Result,
    ids::{EventId, NodeId, RevisionId, ShareId},
    types::{Node, NodeKind, NodeState, Photo, Revision, Share, Volume, VolumeState},
};
use protondrive_events::{EventStream};
use tokio::io::{AsyncRead};
use tracing::instrument;

pub struct ProtonDrive {
    pub(crate) client: ApiClient,
    pub(crate) session: Session,
}

impl std::fmt::Debug for ProtonDrive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtonDrive")
            .field("user_id", &self.session.user_id)
            .finish_non_exhaustive()
    }
}

impl ProtonDrive {
    pub fn builder() -> crate::builder::ProtonDriveBuilder {
        crate::builder::ProtonDriveBuilder::new()
    }

    // ── Volumes & navigation ────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn list_volumes(&self) -> Result<Vec<Volume>> {
        let resp = self.client.list_volumes().await?;
        Ok(resp.volumes.into_iter().map(|v| Volume {
            id: v.volume_id.into(),
            share_id: v.share.share_id.into(),
            state: match v.state {
                2 => VolumeState::Deleted,
                3 => VolumeState::Locked,
                _ => VolumeState::Active,
            },
        }).collect())
    }

    #[instrument(skip(self), fields(share_id = %share, parent = ?parent))]
    pub async fn list_children(
        &self,
        share: &ShareId,
        parent: Option<&NodeId>,
    ) -> Result<Vec<Node>> {
        let resp = self.client.list_children(share, parent).await?;
        Ok(resp.links.into_iter().map(|l| link_dto_to_node(l, share)).collect())
    }

    #[instrument(skip(self), fields(share_id = %share, node_id = %id))]
    pub async fn get_node(&self, share: &ShareId, id: &NodeId) -> Result<Node> {
        let link = self.client.get_link(share, id).await?;
        Ok(link_dto_to_node(link, share))
    }

    // ── Streaming file I/O ──────────────────────────────────────────────────

    #[instrument(skip(self, content), fields(parent = %parent, name = %name))]
    pub async fn upload<R: AsyncRead + Unpin + Send>(
        &self,
        parent: &NodeId,
        name: &str,
        content: R,
        opts: UploadOptions,
    ) -> Result<NodeId> {
        crate::upload::upload(&self.client, parent, name, content, opts).await
    }

    #[instrument(skip(self), fields(node = %node))]
    pub async fn download(&self, node: &NodeId) -> Result<DownloadStream> {
        crate::download::download(&self.client, node).await
    }

    // ── Mutations ───────────────────────────────────────────────────────────

    #[instrument(skip(self), fields(node = %node, new_name = %new_name))]
    pub async fn rename(&self, share: &ShareId, node: &NodeId, new_name: &str) -> Result<()> {
        use protondrive_api::endpoints::nodes::RenameRequest;
        self.client
            .rename_link(share, node, &RenameRequest {
                name: new_name.to_owned(),
                mime_type: None,
            })
            .await
    }

    #[instrument(skip(self), fields(node = %node, new_parent = %new_parent))]
    pub async fn move_node(
        &self,
        share: &ShareId,
        node: &NodeId,
        new_parent: &NodeId,
        new_name: &str,
    ) -> Result<()> {
        use protondrive_api::endpoints::nodes::MoveRequest;
        self.client
            .move_link(share, node, &MoveRequest {
                parent_link_id: new_parent.to_string(),
                name: new_name.to_owned(),
            })
            .await
    }

    #[instrument(skip(self), fields(node = %node))]
    pub async fn trash(&self, share: &ShareId, node: &NodeId) -> Result<()> {
        use protondrive_api::endpoints::nodes::TrashRequest;
        self.client
            .trash_links(share, &TrashRequest {
                link_ids: vec![node.to_string()],
            })
            .await
    }

    #[instrument(skip(self), fields(node = %node))]
    pub async fn delete(&self, share: &ShareId, node: &NodeId) -> Result<()> {
        self.client.delete_link(share, node).await
    }

    // ── Sharing ─────────────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn list_shares(&self) -> Result<Vec<Share>> {
        use protondrive_core::types::ShareFlags;
        let resp = self.client.list_shares().await?;
        Ok(resp.shares.into_iter().map(|s| Share {
            id: s.share_id.into(),
            volume_id: s.volume_id.into(),
            link_id: s.link_id.into(),
            flags: ShareFlags(s.flags),
        }).collect())
    }

    // ── Photos ──────────────────────────────────────────────────────────────

    pub async fn list_photos(&self, _share: &ShareId) -> Result<Vec<Photo>> {
        // Phase 3 deliverable — stub for API surface completeness
        Ok(Vec::new())
    }

    // ── Revisions ───────────────────────────────────────────────────────────

    #[instrument(skip(self), fields(node = %node))]
    pub async fn list_revisions(&self, share: &ShareId, node: &NodeId) -> Result<Vec<Revision>> {
        use protondrive_core::types::RevisionState;
        let resp = self.client.list_revisions(share, node).await?;
        Ok(resp.revisions.into_iter().map(|r| Revision {
            id: r.id.into(),
            node_id: node.clone(),
            size: r.size,
            state: match r.state {
                0 => RevisionState::Draft,
                2 => RevisionState::Superseded,
                3 => RevisionState::Deleted,
                _ => RevisionState::Active,
            },
            created_at: r.create_time,
            manifest_signature: r.manifest_signature,
        }).collect())
    }

    #[instrument(skip(self), fields(node = %node, rev = %rev))]
    pub async fn restore_revision(
        &self,
        share: &ShareId,
        node: &NodeId,
        rev: &RevisionId,
    ) -> Result<()> {
        self.client.restore_revision(share, node, rev).await
    }

    // ── Events ──────────────────────────────────────────────────────────────

    #[instrument(skip(self), fields(share = %share))]
    pub async fn latest_event_id(&self, share: &ShareId) -> Result<EventId> {
        let resp = self.client.get_latest_event_id(share).await?;
        Ok(EventId::new(resp.event_id))
    }

    #[instrument(skip(self), fields(share = %share, since = %since))]
    pub async fn subscribe_events(&self, share: &ShareId, since: EventId) -> Result<EventStream> {
        Ok(EventStream::new(self.client.clone(), share.clone(), since))
    }
}

fn link_dto_to_node(
    l: protondrive_api::endpoints::nodes::LinkDto,
    share: &ShareId,
) -> Node {
    Node {
        id: l.link_id.into(),
        share_id: share.clone(),
        parent_id: l.parent_link_id.map(Into::into),
        name: l.name,
        kind: if l.r#type == 2 { NodeKind::Folder } else { NodeKind::File },
        state: match l.state {
            2 => NodeState::Trashed,
            3 => NodeState::Deleted,
            4 => NodeState::Restoring,
            _ => NodeState::Active,
        },
        created_at: l.create_time,
        modified_at: l.modify_time,
        mime_type: l.mime_type,
        size: l.size,
    }
}
