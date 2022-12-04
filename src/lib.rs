pub mod ext;
pub mod io;

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::error::SendError;
    use tokio::task::JoinHandle;
    use tokio::test;
    use tokio::time::sleep;

    use crate::io;
    use crate::io::file::deserializer::{DeserializationError, Deserializer, Format};
    use crate::io::tcp::client_handler::TcpClientHandle;
    use crate::io::tcp::server::{TcpServer, TcpServerError};

    #[test]
    async fn it_works() {
        let server = TcpServer {
            address: "127.0.0.1".to_string(),
            port: 3456,
        };

        let (a, b) = server.start(Some(&do_it), &do_it_again);

        sleep(Duration::from_secs(15)).await;
    }

    async fn do_it(a: &TcpClientHandle, data: Option<String>) {
        a.send_data("HCasasasasasasasasasasasasasasasas".to_owned())
            .await;
    }

    async fn do_it_again(a: &TcpClientHandle, data: Option<String>) {
        let buffer = vec![0; 1024];
        let string = String::from_utf8_lossy(&buffer);
        let result = a.send_data(data.unwrap()).await;
    }

    // async fn it_works() {
    //     let person = Person { age: 0 };
    //     start(|x| async move { x.yes().await }).await;
    // }
    //
    // async fn sim(person: &Person) {
    //     println!("oleeeeeee")
    // }

    // async fn it_works() {
    //     let server = TcpServer("localhost".to_owned(), 3456);
    //     server.start(
    //         &|a| async {
    //             a.send_data("ola".to_owned()).await;
    //         },
    //         &test12,
    //     );
    // }

    // pub async fn test123(a: &TcpClientHandle) {
    //     t().await
    // }
    //
    // pub async fn t() {}
    //
    // pub async fn test12(a: &TcpClientHandle, b: String) {
    //     t().await
    // }
}
