#![forbid(unsafe_code)]

pub mod builder;
pub mod drive;
pub mod upload;
pub mod download;

pub use builder::ProtonDriveBuilder;
pub use drive::ProtonDrive;

// Re-export core types consumers need
pub use protondrive_auth::{Session, SerializedSession, TwoFactorProvider};
pub use protondrive_core::{
    error::{DriveError, Result},
    ids::{EventId, NodeId, RevisionId, ShareId, VolumeId},
    types::{Node, NodeKind, NodeState, Photo, Revision, Share, Volume},
};
pub use protondrive_events::{DriveEvent, EventAction, EventStream};
