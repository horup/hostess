use hostess::{Server, log::LevelFilter, tokio, uuid::Uuid};
mod server;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let server:Server<server::Server> = Server::new("0.0.0.0:8080");
    server.lobby.write().await.new_host(Uuid::nil());
    let _ = server.spawn().await; 
}
