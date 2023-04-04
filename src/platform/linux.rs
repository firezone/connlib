use boringtun::device::tun::TunSocket;

const tun_name: &str = "wg-firezone";

#[allow(dead_code)]
pub struct Tunnel {
    socket: TunSocket,
}

impl Tunnel {
    pub fn new() -> Result<Self, std::io::Error> {
        match TunSocket::new(tun_name) {
            Ok(socket) => Ok(Self { socket }),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "TunSocket::new() failed",
            )),
        }
    }
}
