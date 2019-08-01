//! Implements web server.

use crate::db::Db;
use crate::templates::*;
use chrono::prelude::*;
use chrono::Duration;
use rouille::{router, Response};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

/// Start the web application.
pub fn start_server(port: u16, db: Arc<Mutex<Db>>) {
    rouille::start_server(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
        move |request| {
            router!(request,
                (GET) (/) => {
                    let measurements = {
                        db.lock().unwrap().select_latest_measurements()
                    };
                    Response::html(base::Base {
                        body: Box::new(index::Index { measurements }),
                    }.to_string())
                },
                (GET) (/sensors/{sensor: String}) => {
                    let (last, _measurements) = {
                        db.lock().unwrap().select_sensor_measurements(&sensor, &(Local::now() - Duration::minutes(5)))
                    };
                    Response::html(base::Base {
                        body: Box::new(sensor::Sensor { last }),
                    }.to_string())
                },
                (GET) (/services) => {
                    Response::html(base::Base {
                        body: Box::new(services::Services { }),
                    }.to_string())
                },
                _ => Response::empty_404(),
            )
        },
    );
}
