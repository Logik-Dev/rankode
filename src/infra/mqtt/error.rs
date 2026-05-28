#[derive(Debug, thiserror::Error)]
pub enum MqttError {
    #[error("failed to serialize notification : {0}")]
    SerializationFailed(#[from] serde_json::Error),

    #[error("failed to publish candidate on Mqtt: {0}")]
    PublishFailed(#[from] rumqttc::ClientError),

    #[error("approval signal content is not valid")]
    InvalidApprovalSignal,
}
