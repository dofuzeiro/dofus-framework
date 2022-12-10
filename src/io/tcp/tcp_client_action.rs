use crate::io::tcp::client_handler::TcpClientTaskHandle;
use crate::io::tcp::tcp_client_action::TcpClientAction::{Connect, Disconnect, SendData};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;

#[derive(Debug)]
pub enum TcpClientAction {
    Connect {
        handle: TcpClientTaskHandle,
    },
    Disconnect {
        handle: TcpClientTaskHandle,
    },
    SendData {
        handle: TcpClientTaskHandle,
        data: String,
    },
}

pub type TcpClientActionSender = mpsc::Sender<TcpClientAction>;
pub type TcpClientActionReceiver = mpsc::Receiver<TcpClientAction>;

pub struct TcpClientActionHandle {
    sender: TcpClientActionSender,
    client_task_handle: TcpClientTaskHandle,
}

impl TcpClientActionHandle {
    pub fn new(sender: TcpClientActionSender, client_task_handle: TcpClientTaskHandle) -> Self {
        TcpClientActionHandle {
            sender,
            client_task_handle,
        }
    }

    pub fn client_disconnect(self) -> Result<(), TrySendError<TcpClientAction>> {
        self.sender.try_send(Disconnect {
            handle: self.client_task_handle,
        })
    }
    pub fn client_connect(self) -> Result<(), TrySendError<TcpClientAction>> {
        self.sender.try_send(Connect {
            handle: self.client_task_handle,
        })
    }
    pub fn client_data(self, data: String) -> Result<(), TrySendError<TcpClientAction>> {
        self.sender.try_send(SendData {
            handle: self.client_task_handle,
            data,
        })
    }
}
