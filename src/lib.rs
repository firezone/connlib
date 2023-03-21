pub fn connect(_portal_url: String, _token: String) -> Result<u32, &'static str> {
    let fd: u32 = 0;
    // TODO: Mock websocket connection, periodically returning updated resources to the client
    Ok(fd)
}

pub fn disconnect() -> Result<bool, &'static str> {
    Ok(true)
}
