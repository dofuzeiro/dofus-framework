use std::future::Future;

use crate::io::tcp::client_handler::TcpClientHandle;

pub trait AsyncHandler<'a> {
    type Fut: Future<Output = ()> + Send + 'a;
    fn call(&self, arg: &'a TcpClientHandle, data: Option<String>) -> Self::Fut;
}

impl<'a, Fu: 'a, F> AsyncHandler<'a> for F
where
    F: Fn(&'a TcpClientHandle, Option<String>) -> Fu,
    Fu: Future<Output = ()> + Send + Sync + 'a,
{
    type Fut = Fu;

    fn call(&self, arg: &'a TcpClientHandle, data: Option<String>) -> Fu {
        self(arg, data)
    }
}
