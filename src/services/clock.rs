use crate::prelude::*;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Clock {
    /// Interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
}

fn default_interval_ms() -> u64 {
    1000
}

impl Clock {
    pub fn spawn(self, service_id: String, tx: Sender) {
        let interval = Duration::from_millis(self.interval_ms);

        tokio::spawn(async move {
            let mut counter = 1;
            loop {
                Message::new(&service_id).value(Value::Counter(counter)).send_to(&tx);
                counter += 1;
                tokio::time::delay_for(interval).await;
            }
        });
    }
}
