use boringtun::device::tun::TunSocket;

#[allow(dead_code)]
pub struct Tunnel {
    socket: TunSocket,
}

impl Tunnel {
    pub fn new() -> Result<Tunnel, std::io::Error> {
        // Loop through all utun interfaces and try to find an unused one
        for index in 0..255 {
            let utun_name = format!("utun{index}");
            if let Ok(socket) = TunSocket::new(utun_name.as_str()) {
                return Ok(Tunnel { socket });
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No more utun interfaces available",
        ))
    }
}
