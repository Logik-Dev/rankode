use anyhow::Result;
use async_trait::async_trait;
use rumqttc::QoS;
use tracing::instrument;

use crate::{
    domain::{MediaFileId, TranscodeEntityPublisher, TranscodedNotification},
    infra::mqtt::{
        client::MqttNotifier,
        error::MqttError,
        mapping::{HaButtonConfig, HaDevice, HaSensorConfig},
    },
};

#[async_trait]
impl TranscodeEntityPublisher for MqttNotifier {
    #[instrument(skip(self, notification), err, fields(file = %notification.file_name))]
    async fn publish_transcoded(&self, notification: &TranscodedNotification) -> Result<()> {
        let id = notification.media_file_id.as_uuid();
        let gain_gb = notification.gain_bytes as f64 / 1e9;
        let state_topic = format!("rankode/transcoded/{id}/state");

        let button = HaButtonConfig {
            name: notification.file_name.clone(),
            unique_id: format!("rankode_{id}"),
            command_topic: "rankode/delete_source",
            payload_press: format!(r#"{{"media_file_id":"{id}"}}"#),
            icon: "mdi:delete",
            device: HaDevice::default(),
        };

        let sensor = HaSensorConfig {
            name: format!("{} gain", notification.file_name),
            unique_id: format!("rankode_{id}_gain"),
            state_topic: state_topic.clone(),
            unit_of_measurement: "GB",
            icon: "mdi:content-save",
            device: HaDevice::default(),
        };

        let button_payload =
            serde_json::to_vec(&button).map_err(MqttError::SerializationFailed)?;
        let sensor_payload =
            serde_json::to_vec(&sensor).map_err(MqttError::SerializationFailed)?;

        self.client
            .publish(
                format!("homeassistant/button/rankode_{id}/config"),
                QoS::AtLeastOnce,
                true,
                button_payload,
            )
            .await
            .map_err(MqttError::PublishFailed)?;

        self.client
            .publish(
                format!("homeassistant/sensor/rankode_{id}_gain/config"),
                QoS::AtLeastOnce,
                true,
                sensor_payload,
            )
            .await
            .map_err(MqttError::PublishFailed)?;

        self.client
            .publish(state_topic, QoS::AtLeastOnce, true, format!("{gain_gb:.2}"))
            .await
            .map_err(MqttError::PublishFailed)?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn unpublish_transcoded(&self, media_file_id: &MediaFileId) -> Result<()> {
        let id = media_file_id.as_uuid();

        self.client
            .publish(
                format!("homeassistant/button/rankode_{id}/config"),
                QoS::AtLeastOnce,
                true,
                vec![],
            )
            .await
            .map_err(MqttError::PublishFailed)?;

        self.client
            .publish(
                format!("homeassistant/sensor/rankode_{id}_gain/config"),
                QoS::AtLeastOnce,
                true,
                vec![],
            )
            .await
            .map_err(MqttError::PublishFailed)?;

        Ok(())
    }
}
