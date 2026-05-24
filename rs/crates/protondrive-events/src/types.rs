use protondrive_core::ids::{EventId, NodeId, ShareId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveEvent {
    pub event_id: EventId,
    pub share_id: ShareId,
    pub action: EventAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventAction {
    /// A node was created or updated.
    NodeUpdated {
        node_id: NodeId,
        payload: serde_json::Value,
    },
    /// A node was deleted.
    NodeDeleted { node_id: NodeId },
    /// A node was trashed.
    NodeTrashed { node_id: NodeId },
    /// A node was restored from trash.
    NodeRestored { node_id: NodeId },
    /// An action we don't recognise — forward-compatibility.
    Unknown { raw_action: u32 },
}

impl EventAction {
    pub fn from_api(action: u32, payload: Option<serde_json::Value>) -> Self {
        match action {
            1 => Self::NodeUpdated {
                node_id: NodeId::new(
                    payload
                        .as_ref()
                        .and_then(|p| p["LinkID"].as_str())
                        .unwrap_or(""),
                ),
                payload: payload.unwrap_or_default(),
            },
            2 => Self::NodeDeleted {
                node_id: NodeId::new(
                    payload
                        .as_ref()
                        .and_then(|p| p["LinkID"].as_str())
                        .unwrap_or(""),
                ),
            },
            3 => Self::NodeTrashed {
                node_id: NodeId::new(
                    payload
                        .as_ref()
                        .and_then(|p| p["LinkID"].as_str())
                        .unwrap_or(""),
                ),
            },
            4 => Self::NodeRestored {
                node_id: NodeId::new(
                    payload
                        .as_ref()
                        .and_then(|p| p["LinkID"].as_str())
                        .unwrap_or(""),
                ),
            },
            n => Self::Unknown { raw_action: n },
        }
    }
}
