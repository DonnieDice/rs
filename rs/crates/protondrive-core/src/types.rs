use crate::ids::{AlbumId, NodeId, RevisionId, ShareId, VolumeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: VolumeId,
    pub share_id: ShareId,
    pub state: VolumeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VolumeState {
    Active,
    Deleted,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub share_id: ShareId,
    pub parent_id: Option<NodeId>,
    pub name: String,
    pub kind: NodeKind,
    pub state: NodeState,
    pub created_at: u64,
    pub modified_at: u64,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeState {
    Active,
    Trashed,
    Deleted,
    Restoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub id: RevisionId,
    pub node_id: NodeId,
    pub size: u64,
    pub state: RevisionState,
    pub created_at: u64,
    pub manifest_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevisionState {
    Draft,
    Active,
    Superseded,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    pub id: ShareId,
    pub volume_id: VolumeId,
    pub link_id: NodeId,
    pub flags: ShareFlags,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShareFlags(u32);

impl ShareFlags {
    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn is_primary(&self) -> bool {
        self.0 & 1 != 0
    }
    pub fn is_locked(&self) -> bool {
        self.0 & 2 != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub node_id: NodeId,
    pub capture_time: Option<u64>,
    pub content_hash: Option<String>,
    pub album_id: Option<AlbumId>,
}
