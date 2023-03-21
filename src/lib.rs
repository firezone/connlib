use rand_core::OsRng;
use x25519_dalek::StaticSecret;

pub fn gen_private_key() -> [u8; 32] {
    StaticSecret::new(OsRng).to_bytes()
}

pub fn connect(
    _portal_url: String,
    _token: String,
    _private_key: &[u8],
) -> Result<u32, &'static str> {
    let fd: u32 = 0;
    // TODO: Mock websocket connection, periodically returning updated resources to the client
    Ok(fd)
}

pub fn disconnect() -> Result<bool, &'static str> {
    Ok(true)
}
