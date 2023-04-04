use platform::tunnel::Tunnel;

mod platform;

#[allow(dead_code)]
pub struct Session {
    tunnel: Tunnel,
}

impl Session {
    pub fn connect(_portal_url: String, _token: String) -> Result<Session, std::io::Error> {
        match Tunnel::new() {
            Ok(tunnel) => Ok(Session { tunnel }),
            Err(e) => Err(e),
        }
    }

    pub fn disconnect(&self) -> bool {
        // 1. Close the websocket connection
        // 2. Free the device handle (UNIX)
        // 3. Close the file descriptor (UNIX)
        // 4. Remove the mapping
        true
    }
}
