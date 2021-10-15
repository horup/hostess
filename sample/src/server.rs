use hostess::{Context, Game};

pub struct Server {

}

impl Game for Server {
    fn new() -> Self {
        Self {
            
        }
    }

    fn tick_rate(&self) -> u64 {
        20
    }

    fn update(&mut self, context:&mut Context) {
        
    }
}