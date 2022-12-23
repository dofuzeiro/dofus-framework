use crate::ddd::application::repository_factory::RepositoryFactory;
use crate::io::tcp::tcp_client_action::TcpClientAction;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActionHandlerError {}

pub trait ActionHandler {
    fn handle<T: RepositoryFactory>(
        &self,
        action: TcpClientAction,
        repository_factory: &T,
    ) -> Result<(), ActionHandlerError>;
}
