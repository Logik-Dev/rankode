use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions};

pub struct MqttNotifier {
    pub(super) client: AsyncClient,
}

impl MqttNotifier {
    pub fn new(host: &str, port: u16) -> Self {
        let mut options = MqttOptions::new("rankode-notifier", host, port);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, mut event_loop) = AsyncClient::new(options, 10);

        tokio::spawn(async move {
            loop {
                if let Err(error) = event_loop.poll().await {
                    tracing::error!("mqtt notifier event loop error: {error}");
                    break;
                }
            }
        });

        Self { client }
    }
}
