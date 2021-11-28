use hostess::{master::Master, server::*, uuid::*};
#[derive(Default)]
struct HelloServer {

}

impl Server for HelloServer {
    fn init(&mut self) -> Config {
        Config {
            tick_rate: 20,
            max_players: 8,
        }
    }

    fn tick(&mut self, ctx:&mut Ctx) {
        println!("Ticking away!");
    }
}



#[tokio::main]
pub async fn main() {
    let mut master = Master::new("127.0.0.1:1234", ServerConstructor::new::<HelloServer>());
    master.new_server(Uuid::default()).await;
    let _ = master.start().await;
}