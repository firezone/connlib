use boringtun::device::tun::TunSocket;

const tun_name: &str = "wg-firezone";

pub struct Tunnel {
    socket: TunSocket,
}

impl Tunnel {
    pub fn new() -> Result<Self, std::io::Error> {
        match TunSocket::new(tun_name) {
            Ok(socket) => Ok(Self { socket }),
            Err(e) => Err(e),
        }
    }
}
