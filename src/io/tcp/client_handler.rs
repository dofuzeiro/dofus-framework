use std::io;

use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::task::JoinHandle;

use crate::io::tcp::async_handler::AsyncHandler;
use crate::io::tcp::client_handler::TcpClientHandlerError::{
    ClientParseError, ClientReadError, ClientWriteError,
};
use crate::io::tcp::server::BUFFER_SIZE;

const CLIENT_BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub enum TcpClientMessage {
    Stop,
    Send { data: String },
}

#[derive(Debug)]
pub struct TcpClientHandle {
    sender: mpsc::Sender<TcpClientMessage>,
}

impl TcpClientHandle {
    pub fn new(sender: mpsc::Sender<TcpClientMessage>) -> Self {
        TcpClientHandle { sender }
    }

    pub async fn send_data(&self, data: String) -> Result<(), TrySendError<TcpClientMessage>> {
        self.sender.try_send(TcpClientMessage::Send { data })
    }
}

pub(crate) struct TcpClientHandler {}

#[derive(Debug, Error)]
pub enum TcpClientHandlerError {
    #[error("Error while writing the data {0} to the client, due to: {1}")]
    ClientWriteError(String, io::Error),
    #[error("Error while trying to parse content from the client")]
    ClientParseError,
    #[error("Error while trying read data from the client")]
    ClientReadError,
}

impl TcpClientHandler {
    pub fn handle_client<OnConnectHandler, OnDataReceivedHandler, OnDisconnectHandler>(
        mut client_stream: TcpStream,
        on_connect: Option<&'static OnConnectHandler>,
        on_data_received: &'static OnDataReceivedHandler,
        on_disconnect: &'static OnDisconnectHandler,
    ) -> JoinHandle<Result<(), TcpClientHandlerError>>
    where
        OnConnectHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
        OnDataReceivedHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
        OnDisconnectHandler: for<'a> AsyncHandler<'a> + Send + Sync + 'static,
    {
        let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
        tokio::spawn(async move {
            let (reader, writer) = client_stream.split();
            let client_handle = TcpClientHandle { sender };
            if let Some(value) = on_connect {
                value.call(&client_handle, None).await
            }
            select! {
                res = Self::listen_to_messages(receiver, writer) => {res?}
                res = Self::listen_to_client(&client_handle, reader, on_data_received, on_disconnect) => {res?}
            }
            Ok::<(), TcpClientHandlerError>(())
        })
    }

    async fn listen_to_client<'a, OnDataReceivedHandler, OnDisconnectHandler>(
        client_handle: &'a TcpClientHandle,
        mut reader: ReadHalf<'_>,
        on_data_received: &'a OnDataReceivedHandler,
        on_disconnect: &'a OnDisconnectHandler,
    ) -> Result<(), TcpClientHandlerError>
    where
        OnDataReceivedHandler: AsyncHandler<'a> + Send + Sync + 'static,
        OnDisconnectHandler: AsyncHandler<'a> + Send + Sync + 'static,
    {
        let mut buffer = [0u8; CLIENT_BUFFER_SIZE];
        loop {
            match reader.read(&mut buffer).await {
                Ok(bytes_read) if bytes_read == 0 => {
                    on_disconnect.call(client_handle, None).await;
                    return Ok(());
                }
                Ok(bytes_read) => {
                    let data_as_string = String::from_utf8(Vec::from(&buffer[0..bytes_read]))
                        .map_err(|_| ClientParseError)?;
                    on_data_received
                        .call(client_handle, Some(data_as_string))
                        .await
                }
                Err(_) => return Err(ClientReadError),
            }
        }
    }

    async fn listen_to_messages(
        mut receiver: mpsc::Receiver<TcpClientMessage>,
        mut writer: WriteHalf<'_>,
    ) -> Result<(), TcpClientHandlerError> {
        loop {
            match receiver.recv().await {
                None => return Ok(()),
                Some(TcpClientMessage::Stop) => return Ok(()),
                Some(TcpClientMessage::Send { data }) => writer
                    .write_all(data.as_bytes())
                    .await
                    .map_err(|e| ClientWriteError(data, e)),
            }?
        }
    }
}
