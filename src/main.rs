//! # Getting started
//!
//! Grab a release from [GitHub](https://github.com/eigenein/my-iot-rs/releases) for your architecture
//!
//! or, install it via `cargo`:
//!
//! ```sh
//! cargo install my-iot
//! ```
//!
//! or, clone the repo and build it manually:
//!
//! ```sh
//! git clone https://github.com/eigenein/my-iot-rs.git
//! cd my-iot-rs
//! make
//! sudo make install
//! ```
//!
//! Then, you'll need to create a configuration file `settings.yml`. It must contain exactly one object,
//! please read the [`settings`](settings/index.html) documentation.
//!
//! ## OpenSSL
//!
//! Compiling My IoT may require you to install `pkg-config` and OpenSSL. Most likely, `libssl-dev`
//! is not installed by default on Raspbian.
//!
//! See https://docs.rs/openssl/0.10.24/openssl/#automatic for more information.
//!
//! ## File capabilities
//!
//! If you're not using `make`, you may need to manually set capabilities on the produced binary:
//!
//! ```sh
//! setcap cap_net_raw+ep /usr/local/bin/my-iot
//! ```
//!
//! This is needed to use some low-level protocols (for instance, ICMP) as a non-root user.

use crate::db::Db;
use crate::reading::Reading;
use crate::settings::Settings;
use crate::threading::ArcMutex;
use crossbeam_channel::{bounded, Receiver, Sender};
use failure::Error;
use log::{debug, info};
use std::sync::{Arc, Mutex};

pub mod consts;
pub mod db;
pub mod logging;
pub mod reading;
pub mod receiver;
pub mod services;
pub mod settings;
pub mod templates;
pub mod threading;
pub mod value;
pub mod web;

type Result<T> = std::result::Result<T, Error>;

/// Entry point.
fn main() -> Result<()> {
    logging::init();

    clap::App::new("My IoT")
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about(clap::crate_description!())
        .get_matches();

    info!("Reading settings…");
    let settings = settings::read()?; // TODO: CLI parameter.
    debug!("Settings: {:?}", &settings);

    info!("Opening database…");
    let db = Arc::new(Mutex::new(Db::new("my-iot.sqlite3")?));

    info!("Starting services…");
    let (tx, rx) = bounded(0);
    spawn_services(&settings, &db, &tx, &rx)?;

    info!("Starting readings receiver…");
    receiver::start(rx.clone(), db.clone())?;

    info!("Starting web server on port {}…", settings.http_port);
    web::start_server(settings, db.clone())
}

/// Spawn all configured services.
fn spawn_services(settings: &Settings, db: &ArcMutex<Db>, tx: &Sender<Reading>, rx: &Receiver<Reading>) -> Result<()> {
    for (service_id, settings) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, settings);
        services::new(service_id, settings)?.spawn(db.clone(), tx.clone(), rx.clone())?;
    }
    Ok(())
}
