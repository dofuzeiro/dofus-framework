use std::fmt::Formatter;
use std::io;

use thiserror::Error;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::io::tcp::async_handler::AsyncHandler;
use crate::io::tcp::client_handler::TcpClientHandler;
use crate::io::tcp::server::TcpServerError::{AcceptClientError, BindingError, SendMessageError};
use crate::io::tcp::server::TcpServerMessage::Stop;

pub(crate) const BUFFER_SIZE: usize = 8;

pub struct TcpServer {
    pub(crate) address: String,
    pub(crate) port: u16,
}

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
}

pub struct TcpServerHandle(mpsc::Sender<TcpServerMessage>);

impl TcpServer {
    pub fn start<OnConnectHandler, OnDataReceivedHandler, OnDisconnectHandler>(
        self,
        on_connect: Option<&'static OnConnectHandler>,
        on_data_received: &'static OnDataReceivedHandler,
        on_disconnect: &'static OnDisconnectHandler,
    ) -> (TcpServerHandle, JoinHandle<Result<(), TcpServerError>>)
    where
        OnConnectHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
        OnDataReceivedHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
        OnDisconnectHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
    {
        let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
        let join_handle = tokio::spawn(async move {
            select! {
                res = self.listen_to_clients(on_connect, on_data_received, on_disconnect) => {
                    info!("Server just stopped listening to clients");
                    res?;
                }
                Some(_) = self.listen_to_messages(receiver) => {
                    info!("Server just stopped listening to messages");
                }
            }
            Ok::<(), TcpServerError>(())
        });
        (TcpServerHandle::new(sender), join_handle)
    }

    async fn listen_to_clients<OnConnectHandler, OnDataReceivedHandler, OnDisconnectHandler>(
        &self,
        on_connect: Option<&'static OnConnectHandler>,
        on_data_received: &'static OnDataReceivedHandler,
        on_disconnect: &'static OnDisconnectHandler,
    ) -> Result<(), TcpServerError>
    where
        OnConnectHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
        OnDataReceivedHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
        OnDisconnectHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
    {
        let binding = format!("{}:{}", self.address, self.port);
        let listener = TcpListener::bind(&binding)
            .await
            .map_err(|e| BindingError {
                source: e,
                address: binding,
            })?;
        loop {
            let (client_stream, _) = listener.accept().await.map_err(AcceptClientError)?;
            info!("A new client has just connected");
            TcpClientHandler::handle_client(
                client_stream,
                on_connect,
                on_data_received,
                on_disconnect,
            );
        }
    }

    async fn listen_to_messages(
        &self,
        mut receiver: mpsc::Receiver<TcpServerMessage>,
    ) -> Option<()> {
        match receiver.recv().await {
            Some(Stop) => Some(()),
            Some(TcpServerMessage::Other) => None,
            None => Some(()),
        }
    }
}

impl TcpServerHandle {
    pub fn new(sender: mpsc::Sender<TcpServerMessage>) -> Self {
        TcpServerHandle(sender)
    }
    pub async fn stop(&self) -> Result<(), TcpServerError> {
        self.0.try_send(Stop).map_err(|_| SendMessageError(Stop))
    }
}
