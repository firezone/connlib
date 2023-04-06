use boringtun::device::tun::TunSocket;

const TUN_NAME: &str = "wg-firezone";

#[allow(dead_code)]
pub struct Tunnel {
    socket: TunSocket,
}

impl Tunnel {
    pub fn new() -> Result<Self, std::io::Error> {
        match TunSocket::new(TUN_NAME) {
            Ok(socket) => Ok(Self { socket }),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "TunSocket::new() failed",
            )),
        }
    }
}
