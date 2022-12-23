use crate::ddd::application::repository_factory::RepositoryFactory;
use crate::dofus::realm::application::action_handler::ActionHandler;
use crate::dofus::realm::application::realm::RealmError::ServerError;
use crate::dofus::realm::application::realm_config::RealmConfig;
use crate::dofus::realm::application::realm_handle::{RealmHandle, RealmMessage};
use crate::io::tcp::server::{TcpServer, BUFFER_SIZE};
use thiserror::Error;
use tokio::select;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum RealmError {
    #[error("Error while processing client request")]
    ClientHandleError,
    #[error("The server just returned the following error")]
    ServerError,
}

pub fn start<T: RepositoryFactory + Send + 'static, E: ActionHandler + Send + 'static>(
    config: RealmConfig,
    repository_factory: T,
    action_handler: E,
) -> RealmHandle {
    let RealmConfig {
        name: _,
        address,
        port,
    } = config;
    let (realm_sender, mut realm_receiver) = mpsc::channel(BUFFER_SIZE);
    let realm_join = tokio::spawn(async move {
        let (tcp_client_sender, mut tcp_client_receiver) = mpsc::channel(BUFFER_SIZE);
        let (server_handle, mut server_join) = TcpServer::start(tcp_client_sender, address, port);
        let mut error: Result<(), RealmError> = Ok(());
        loop {
            select! {
                Some(action) = tcp_client_receiver.recv() => { // handle client actions (e.g connect, dc or data sent)
                    if let Err(_msg) =  &action_handler.handle(action, &repository_factory) {
                        error = Err(RealmError::ClientHandleError);
                        break
                }}
                Some(RealmMessage::Stop) = realm_receiver.recv() => {break} // listen to messages of the realm handle
                server_result = &mut server_join => {
                    if server_result.is_err() {
                        error = Err(ServerError);
                        break;
                    }
                }
            }
        }
        let _ = server_handle.stop(); // lets signal the tcp server to stop
        match server_join.await {
            // wait for server tcp server to gracefully close
            Err(_) => error = Err(ServerError),     // join error
            Ok(Err(_)) => error = Err(ServerError), // tcp server error
            _ => {}
        }
        error
    });
    RealmHandle::new(realm_sender, realm_join)
}
