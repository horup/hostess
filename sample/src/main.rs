use hostess::{Game, GameConstructor, Server, log::LevelFilter, tokio, uuid::Uuid};
mod server;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let f = || {
        let boxed:Box<dyn Game> = Box::new(server::Server::new());
        return boxed;
    };
    let constructor = GameConstructor::new(Box::new(f));
    let server:Server = Server::new("0.0.0.0:8080", constructor.clone());
    server.lobby.write().await.new_host(Uuid::nil(), constructor);
    let _ = server.spawn().await; 
}