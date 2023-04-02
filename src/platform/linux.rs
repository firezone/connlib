use boringtun::device::tun::TunSocket;

pub struct Tunnel {
    socket: TunSocket,
}

impl Tunnel {
    pub fn new() -> Result<Self, std::io::Error> {
        match TunSocket::new() {
            Ok(socket) => Ok(Self { socket }),
            Err(e) => Err(e),
        }
    }
}
