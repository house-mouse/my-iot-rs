//! Implements generic `Service` trait.

use crate::prelude::*;
use crate::settings::{Service, Settings};

/// Spawn all the configured services.
pub fn spawn_all(settings: &Settings, tx: Sender) -> Result<()> {
    for (service_id, service) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, service);
        let service_id = service_id.clone();
        let tx = tx.clone();
        match service.clone() {
            Service::Buienradar(buienradar) => buienradar.spawn(service_id, tx)?,
            Service::Clock(clock) => clock.spawn(service_id, tx),
            Service::Lua(lua) => lua.spawn(service_id, tx, &settings.services)?,
            Service::Solar(solar) => solar.spawn(service_id, tx),
            Service::Tado(tado) => tado.spawn(service_id, tx)?,
            Service::Telegram(telegram) => telegram.spawn(service_id, tx)?,
        };
    }
    Ok(())
}
