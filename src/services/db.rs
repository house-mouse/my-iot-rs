use crate::prelude::*;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60);

pub struct Db;

impl Db {
    pub fn spawn(self, db: Arc<Mutex<Connection>>, tx: Sender) {
        tokio::spawn(async move {
            loop {
                if let Err(error) = Self::refresh(&db, &tx) {
                    error!("Failed to refresh: {}", error.to_string());
                }
                tokio::time::delay_for(INTERVAL).await;
            }
        });
    }

    fn refresh(db: &Arc<Mutex<Connection>>, tx: &Sender) -> Result<()> {
        let db = db.lock().unwrap();
        Message::new("db::size")
            .value(Value::DataSize(db.select_size()?))
            .sensor_title("Database Size".to_string())
            .room_title("System".to_string())
            .send_to(&tx);
        Message::new("db::sensor_count")
            .value(Value::Counter(db.select_sensor_count()?))
            .sensor_title("Sensor Count")
            .room_title("System".to_string())
            .send_to(&tx);
        Message::new("db::reading_count")
            .value(Value::Counter(db.select_reading_count()?))
            .sensor_title("Reading Count")
            .room_title("System".to_string())
            .send_to(&tx);
        Ok(())
    }
}
