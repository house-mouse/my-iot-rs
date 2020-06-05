use crate::core::message::Type;
use crate::prelude::*;
use spa::{calc_sunrise_and_set, SunriseAndSet};
use std::time::Duration;
use uom::si::f64::*;
use uom::si::*;

/// Emits durations to and after sunrise and sunset.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Solar {
    /// Message interval in milliseconds.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,

    #[serde(default)]
    pub room_title: Option<String>,

    pub secrets: Secrets,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    /// Latitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-90.0` to `90.0`.
    pub latitude: f64,

    /// Longitude in [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System) system, ranging from `-180.0` to `180.0`
    pub longitude: f64,
}

/// Defaults to one minute.
fn default_interval_ms() -> u64 {
    60000
}

impl Solar {
    pub fn spawn(self, service_id: String, tx: Sender) {
        let interval = Duration::from_millis(self.interval_ms);

        tokio::spawn(async move {
            loop {
                if let Err(error) = self.refresh(&service_id, &tx) {
                    error!("[{}] Failed to refresh: {}", service_id, error.to_string());
                }
                tokio::time::delay_for(interval).await;
            }
        });
    }

    fn refresh(&self, service_id: &str, tx: &Sender) -> Result<()> {
        let now = Utc::now();
        match calc_sunrise_and_set(now, self.secrets.latitude, self.secrets.longitude)? {
            SunriseAndSet::Daylight(sunrise, sunset) => {
                if now < sunrise {
                    Message::new(format!("{}::before::sunrise", service_id))
                        .type_(Type::ReadSnapshot)
                        .sensor_title("Time Before Sunrise")
                        .optional_room_title(self.room_title.clone())
                        .value(Time::new::<time::millisecond>((sunrise - now).num_milliseconds() as f64))
                        .send_to(&tx);
                }
                if now < sunset {
                    Message::new(format!("{}::before::sunset", service_id))
                        .type_(Type::ReadSnapshot)
                        .sensor_title("Time Before Sunset")
                        .optional_room_title(self.room_title.clone())
                        .value(Time::new::<time::millisecond>((sunset - now).num_milliseconds() as f64))
                        .send_to(&tx);
                }
                if sunrise < now {
                    Message::new(format!("{}::after::sunrise", service_id))
                        .type_(Type::ReadSnapshot)
                        .sensor_title("Time After Sunrise")
                        .optional_room_title(self.room_title.clone())
                        .value(Time::new::<time::millisecond>((now - sunrise).num_milliseconds() as f64))
                        .send_to(&tx);
                }
                if sunset < now {
                    Message::new(format!("{}::after::sunset", service_id))
                        .type_(Type::ReadSnapshot)
                        .sensor_title("Time After Sunset")
                        .optional_room_title(self.room_title.clone())
                        .value(Time::new::<time::millisecond>((now - sunset).num_milliseconds() as f64))
                        .send_to(&tx);
                }
            }
            SunriseAndSet::PolarDay => {
                Message::new(format!("{}::polar_day", service_id))
                    .type_(Type::ReadNonLogged)
                    .optional_room_title(self.room_title.clone())
                    .send_to(&tx);
            }
            SunriseAndSet::PolarNight => {
                Message::new(format!("{}::polar_night", service_id))
                    .type_(Type::ReadNonLogged)
                    .optional_room_title(self.room_title.clone())
                    .send_to(&tx);
            }
        }
        Ok(())
    }
}
