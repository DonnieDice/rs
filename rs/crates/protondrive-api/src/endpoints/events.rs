use crate::client::ApiClient;
use protondrive_core::{
    error::Result,
    ids::{EventId, ShareId, VolumeId},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EventsResponse {
    #[serde(rename = "EventID")]
    pub event_id: String,
    #[serde(rename = "More")]
    pub more: u8,
    #[serde(rename = "Events")]
    pub events: Vec<EventDto>,
}

#[derive(Debug, Deserialize)]
pub struct EventDto {
    #[serde(rename = "EventID")]
    pub event_id: String,
    #[serde(rename = "Action")]
    pub action: u32,
    #[serde(rename = "Link")]
    pub link: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct LatestEventResponse {
    #[serde(rename = "EventID")]
    pub event_id: String,
}

impl ApiClient {
    pub async fn get_latest_core_event_id(&self) -> Result<LatestEventResponse> {
        self.get("/core/v4/events/latest").await
    }

    pub async fn get_core_events(&self, since: &EventId) -> Result<EventsResponse> {
        self.get(&format!("/core/v5/events/{since}")).await
    }

    pub async fn get_latest_volume_event_id(
        &self,
        volume_id: &VolumeId,
    ) -> Result<LatestEventResponse> {
        self.get(&latest_volume_event_route(volume_id)).await
    }

    pub async fn get_volume_events(
        &self,
        volume_id: &VolumeId,
        since: &EventId,
    ) -> Result<EventsResponse> {
        self.get(&volume_events_route(volume_id, since)).await
    }

    pub async fn get_latest_event_id(&self, share_id: &ShareId) -> Result<LatestEventResponse> {
        self.get(&format!("/drive/shares/{share_id}/events/latest"))
            .await
    }

    pub async fn get_events(&self, share_id: &ShareId, since: &EventId) -> Result<EventsResponse> {
        self.get(&format!("/drive/shares/{share_id}/events/{since}"))
            .await
    }
}

fn latest_volume_event_route(volume_id: &VolumeId) -> String {
    format!("/drive/volumes/{volume_id}/events/latest")
}

fn volume_events_route(volume_id: &VolumeId, since: &EventId) -> String {
    format!("/drive/v2/volumes/{volume_id}/events/{since}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_route_builders_match_current_drive_api() {
        let volume = VolumeId::from("volume");
        let event = EventId::from("event");

        assert_eq!(
            latest_volume_event_route(&volume),
            "/drive/volumes/volume/events/latest"
        );
        assert_eq!(
            volume_events_route(&volume, &event),
            "/drive/v2/volumes/volume/events/event"
        );
    }
}
