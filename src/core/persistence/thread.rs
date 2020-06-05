use crate::prelude::*;

/// Spawn the persistence task.
pub fn spawn(db: Arc<Mutex<Connection>>, mut rx: Receiver) {
    info!("Spawning readings persistence…");

    tokio::spawn(async move {
        loop {
            let message = Message::receive_from(&mut rx).await;
            if let Err(error) = upsert_message(&message, &db) {
                error!("{}: {:?}", error, &message);
            }
        }
    });
}

fn upsert_message(message: &Message, db: &Arc<Mutex<Connection>>) -> Result<()> {
    info!(
        "{}: {:?} {:?}",
        &message.sensor.id, &message.type_, &message.reading.value
    );
    debug!("{:?}", &message);
    // TODO: handle `ReadSnapshot` properly.
    if message.type_ == MessageType::ReadLogged || message.type_ == MessageType::ReadSnapshot {
        message.upsert_into(&db.lock().unwrap())?;
    }
    Ok(())
}
