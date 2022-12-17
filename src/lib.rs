pub mod ddd;
pub mod ext;
pub mod io;
pub use tokio;

#[cfg(test)]
mod tests {
    use tokio::test;

    #[test]
    async fn it_works() {}
}
