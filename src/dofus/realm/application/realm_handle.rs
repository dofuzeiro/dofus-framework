use crate::dofus::realm::application::realm::RealmError;
use crate::dofus::realm::application::realm_handle::RealmMessage::Stop;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub enum RealmMessage {
    Stop,
}

pub struct RealmHandle {
    sender: Sender<RealmMessage>,
    join: JoinHandle<Result<(), RealmError>>,
}

impl RealmHandle {
    pub fn new(sender: Sender<RealmMessage>, join: JoinHandle<Result<(), RealmError>>) -> Self {
        RealmHandle { sender, join }
    }

    pub async fn stop(self) {
        let _ = self.sender.try_send(Stop);
        let _ = self.join.await;
    }
}
