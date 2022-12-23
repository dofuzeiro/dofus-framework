use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::fmt::Formatter;
use std::io;

use thiserror::Error;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::io::tcp::client_handler::{TcpClientTask, TcpClientTaskError};
use crate::io::tcp::server::TcpServerError::{BindingError, SendMessageError};
use crate::io::tcp::server::TcpServerMessage::Stop;
use crate::io::tcp::tcp_client_action::TcpClientAction;

pub(crate) const BUFFER_SIZE: usize = 8;

pub struct TcpServer {}

#[derive(Debug)]
pub enum TcpServerMessage {
    Stop,
    Other,
}

impl std::fmt::Display for TcpServerMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stop => {
                write!(f, "Stop")
            }
            TcpServerMessage::Other => {
                write!(f, "Other")
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum TcpServerError {
    #[error("Error while trying to bind to the given address {address:?}, due to: {source:?}")]
    BindingError { address: String, source: io::Error },
    #[error("Error while accepting a new tcp client, due to: {0}")]
    AcceptClientError(io::Error),
    #[error("Error while sending the following command to the server: {0}")]
    SendMessageError(TcpServerMessage),
    #[error("An error occurred while joining the client tasks")]
    JoiningError,
    #[error("A client task exited with the following error: {0}")]
    ClientTaskError(TcpClientTaskError),
}

pub struct TcpServerHandle(mpsc::Sender<TcpServerMessage>);

impl TcpServer {
    pub fn start(
        sender: mpsc::Sender<TcpClientAction>,
        address: String,
        port: u16,
    ) -> (TcpServerHandle, JoinHandle<Result<(), TcpServerError>>) {
        let (tcp_server_sender, mut receiver) = mpsc::channel(BUFFER_SIZE);
        let join_handle = tokio::spawn(async move {
            let (listener, mut client_tasks) = Self::bind_address(address, port).await?;
            let mut result: Result<(), TcpServerError> = Ok(());
            loop {
                select! {
                     accept_result = listener.accept() => { // Listen to new clients.
                        let (stream, _socket) = match accept_result {
                            Ok((stream, _socket)) => (stream, _socket),
                            Err(msg) => { // Stop server immediately if accept returns an error
                                result = Err(TcpServerError::AcceptClientError(msg));
                                break
                            }
                        };
                        client_tasks.push(TcpClientTask::handle_client(stream, sender.clone()));
                        // Create a TcpClientTask (which spawns a new task, to handle the client connection)
                    }
                    Some(_finished_client_task) = client_tasks.next() => { // Listen to exiting tcp client tasks
                        //TODO handle client task exiting
                    }
                    Some(TcpServerMessage::Stop) = receiver.recv() => { // Listen to tcpserver handle messages (e.g stop server)
                        info!("Server just stopped listening to messages");
                        break
                    }
                }
            }
            //TODO do we need to signal the client tasks to stop?
            futures::future::join_all(client_tasks).await; // Wait for all child tasks to terminate
            result
        });
        (TcpServerHandle::new(tcp_server_sender), join_handle)
    }

    async fn bind_address(
        address: String,
        port: u16,
    ) -> Result<
        (
            TcpListener,
            FuturesUnordered<JoinHandle<Result<(), TcpClientTaskError>>>,
        ),
        TcpServerError,
    > {
        let binding = format!("{}:{}", address, port);
        let listener = TcpListener::bind(&binding)
            .await
            .map_err(|e| BindingError {
                source: e,
                address: binding,
            })?;
        let client_tasks: FuturesUnordered<JoinHandle<Result<(), TcpClientTaskError>>> =
            FuturesUnordered::new();
        Ok((listener, client_tasks))
    }
}

impl TcpServerHandle {
    pub fn new(sender: mpsc::Sender<TcpServerMessage>) -> Self {
        TcpServerHandle(sender)
    }
    pub fn stop(&self) -> Result<(), TcpServerError> {
        self.0.try_send(Stop).map_err(|_| SendMessageError(Stop))
    }
}
