use signals::PlayerSignal;
use std::{io::BufReader, net::TcpStream};

pub struct Client {
    stream: BufReader<TcpStream>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let stream = BufReader::new(stream);
        Self { stream }
    }
}

impl Client {
    pub fn read_signal(&mut self) -> Result<PlayerSignal, bincode::Error> {
        bincode::deserialize_from(&mut self.stream)
    }
}
