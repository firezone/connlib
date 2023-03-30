use boringtun::device::tun::TunSocket;

/// Firezone explicitly ties the WireGuard tunnel interface lifetime to
/// the Control place connection lifetime under a single struct called Session.
pub struct Session {
    socket: TunSocket,
    // TODO: WebSocket connection handle etc
}

impl Session {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub fn connect(_portal_url: String, _token: String) -> Result<Session, std::io::Error> {
        for index in 0..255 {
            let utun_name = format!("utun{index}");
            if let Ok(socket) = TunSocket::new(utun_name.as_str()) {
                println!("Connected to portal");
                return Ok(Session { socket });
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No more utun interfaces available",
        ))
    }

    pub fn disconnect(&self) -> bool {
        // 1. Close the websocket connection
        // 2. Free the device handle (UNIX)
        // 3. Close the file descriptor (UNIX)
        // 4. Remove the mapping

        // TODO: Close the socket
        println!("Closed the websocket connection");

        true
    }
}
