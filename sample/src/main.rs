use hostess::{log::LevelFilter, Server, tokio};
mod server;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let server:Server<server::Server> = Server::new("0.0.0.0:8080");
    let _ = server.spawn().await;
}
