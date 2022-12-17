use std::io;

use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::task::JoinHandle;

use crate::io::tcp::client_handler::TcpClientTaskError::{
    ClientConnectError, ClientDataError, ClientDisconnectError, ClientParseError, ClientReadError,
    ClientWriteError,
};
use crate::io::tcp::server::BUFFER_SIZE;
use crate::io::tcp::tcp_client_action::{TcpClientActionHandle, TcpClientActionSender};

const CLIENT_BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub enum TcpClientTaskMessage {
    Stop,
    Send { data: String },
}

#[derive(Debug, Clone)]
pub struct TcpClientTaskHandle {
    sender: mpsc::Sender<TcpClientTaskMessage>,
}

impl TcpClientTaskHandle {
    pub fn new(sender: mpsc::Sender<TcpClientTaskMessage>) -> Self {
        TcpClientTaskHandle { sender }
    }

    pub fn send_data(&self, data: String) -> Result<(), TrySendError<TcpClientTaskMessage>> {
        self.sender.try_send(TcpClientTaskMessage::Send { data })
    }

    pub fn stop(&self) -> Result<(), TrySendError<TcpClientTaskMessage>> {
        self.sender.try_send(TcpClientTaskMessage::Stop)
    }
}

pub(crate) struct TcpClientTask {}

#[derive(Debug, Error)]
pub enum TcpClientTaskError {
    #[error("Error while writing the data {0} to the client, due to: {1}")]
    ClientWriteError(String, io::Error),
    #[error("Error while trying to parse content from the client")]
    ClientParseError,
    #[error("Error while trying to read data from the client")]
    ClientReadError,
    #[error("Error while trying to send client connect action")]
    ClientConnectError,
    #[error("Error while trying to send client disconnect action")]
    ClientDisconnectError,
    #[error("Error while trying to send client send data action")]
    ClientDataError,
}

impl TcpClientTask {
    pub fn handle_client(
        mut client_stream: TcpStream,
        tcp_client_action_sender: TcpClientActionSender,
    ) -> JoinHandle<Result<(), TcpClientTaskError>> {
        let (tcp_client_task_sender, tcp_client_task_receiver) = mpsc::channel(BUFFER_SIZE);
        tokio::spawn(async move {
            let (reader, writer) = client_stream.split();
            select! {
                res = Self::listen_to_messages(tcp_client_task_receiver, writer) => {res}
                res = Self::listen_to_client(reader, tcp_client_action_sender, TcpClientTaskHandle::new(tcp_client_task_sender)) => {res}
            }
        })
    }

    async fn listen_to_client(
        mut reader: ReadHalf<'_>,
        tcp_client_action_sender: TcpClientActionSender,
        tcp_client_task_handle: TcpClientTaskHandle,
    ) -> Result<(), TcpClientTaskError> {
        TcpClientActionHandle::new(
            tcp_client_action_sender.clone(),
            tcp_client_task_handle.clone(),
        )
        .client_connect()
        .map_err(|_| ClientConnectError)?;
        let mut buffer = [0u8; CLIENT_BUFFER_SIZE];
        loop {
            match reader.read(&mut buffer).await {
                Ok(bytes_read) if bytes_read == 0 => {
                    TcpClientActionHandle::new(
                        tcp_client_action_sender.clone(),
                        tcp_client_task_handle.clone(),
                    )
                    .client_disconnect()
                    .map_err(|_| ClientDisconnectError)?;
                    return Ok(());
                }
                Ok(bytes_read) => {
                    let data_as_string = String::from_utf8(Vec::from(&buffer[0..bytes_read]))
                        .map_err(|_| ClientParseError)?;
                    TcpClientActionHandle::new(
                        tcp_client_action_sender.clone(),
                        tcp_client_task_handle.clone(),
                    )
                    .client_data(data_as_string)
                    .map_err(|_| ClientDataError)?;
                }
                Err(_) => return Err(ClientReadError),
            }
        }
    }

    async fn listen_to_messages(
        mut receiver: mpsc::Receiver<TcpClientTaskMessage>,
        mut writer: WriteHalf<'_>,
    ) -> Result<(), TcpClientTaskError> {
        loop {
            match receiver.recv().await {
                None => return Ok(()),
                Some(TcpClientTaskMessage::Stop) => return Ok(()),
                Some(TcpClientTaskMessage::Send { data }) => writer
                    .write_all(data.as_bytes())
                    .await
                    .map_err(|e| ClientWriteError(data, e)),
            }?
        }
    }
}
