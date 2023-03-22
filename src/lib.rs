/// Firezone explicitly ties the WireGuard tunnel interface lifetime to
/// the Control place connection lifetime under a single struct called Session.
pub struct Session {
    fd: u32,
    handle: u32,
}

pub fn connect(_portal_url: String, _token: String) -> Result<Session, &'static str> {
    let fd = 9999;
    let handle = fd + 1;

    // 1. Find a free file descriptor
    // 2. Allocate a device handle
    // 3. Save this mapping for looking up later
    // 4. Connect the WebSocket
    println!("Found a free file descriptor: {fd}");
    println!("Allocated device handle: {handle}");

    // TODO: Mock websocket connection, periodically returning updated resources to the client
    println!("Connected to portal");

    Ok(Session { fd, handle })
}

pub fn disconnect(session: Session) -> Result<bool, &'static str> {
    // 1. Close the websocket connection
    // 2. Free the device handle
    // 3. Close the file descriptor
    // 4. Remove the mapping
    println!("Closed the websocket connection");
    println!("Freed the device handle {}", session.handle);
    println!("Closed the file descriptor {}", session.fd);
    println!("Removed the mapping");

    Ok(true)
}