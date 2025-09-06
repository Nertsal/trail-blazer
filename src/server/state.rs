use super::*;

struct Config {}

pub struct Client {
    pub sender: Box<dyn geng::net::Sender<ServerMessage>>,
}

pub struct ServerState {
    pub timer: Timer,
    pub next_id: ClientId,
    pub config: Config,
    pub clients: HashMap<ClientId, Client>,
    pub map_size: vec2<ICoord>,
}

impl ServerState {
    pub const TICKS_PER_SECOND: f32 = 1.0;

    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
            next_id: 1,
            config: Config {},
            clients: HashMap::new(),
            map_size: vec2(10, 10),
        }
    }

    pub fn tick(&mut self) {}

    pub fn get_setup(&self) -> Setup {
        Setup {
            map_size: self.map_size,
        }
    }
}
