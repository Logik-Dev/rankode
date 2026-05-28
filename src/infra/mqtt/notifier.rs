use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;

use crate::{
    domain::{ApprovalNotifier, CandidateNotification},
    infra::mqtt::{client::MqttNotifier, error::MqttError, mapping::CandidatePayload},
};

#[async_trait]
impl ApprovalNotifier for MqttNotifier {
    #[instrument(skip(self, notification), err, fields(filename = %notification.file_name))]
    async fn notify_candidate(&self, notification: CandidateNotification) -> Result<()> {
        let payload = serde_json::to_vec::<CandidatePayload>(&notification.into())
            .map_err(MqttError::SerializationFailed)?;

        self.client
            .publish(
                "rankode/candidate",
                rumqttc::QoS::ExactlyOnce,
                false,
                payload,
            )
            .await
            .map_err(MqttError::PublishFailed)?;

        Ok(())
    }
}
