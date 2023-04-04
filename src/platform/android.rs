#[allow(dead_code)]
pub struct Tunnel {
    fd: i32,
}

impl Tunnel {
    pub fn new() -> Result<Self, std::io::Error> {
        // On android, the file descriptor is passed from the VPN service. We'll need to accept it
        // or set it later.
        Ok(Self { fd: -1 })
    }
}
