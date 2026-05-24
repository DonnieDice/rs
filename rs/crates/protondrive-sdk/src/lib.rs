#![forbid(unsafe_code)]

pub mod builder;
pub mod download;
pub mod drive;
pub mod upload;

pub use builder::ProtonDriveBuilder;
pub use drive::ProtonDrive;

// Re-export core types consumers need
pub use protondrive_auth::{SerializedSession, Session, TwoFactorProvider};
pub use protondrive_core::{
    error::{DriveError, Result},
    ids::{EventId, NodeId, RevisionId, ShareId, VolumeId},
    types::{Node, NodeKind, NodeState, Photo, Revision, Share, Volume},
};
pub use protondrive_events::{DriveEvent, EventAction, EventStream};
