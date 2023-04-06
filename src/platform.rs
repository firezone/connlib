// Tunnel management for Linux
#[cfg(target_os = "linux")]
#[path = "platform/linux.rs"]
pub mod tunnel;

// Tunnel management for macOS and iOS
#[cfg(any(target_os = "ios", target_os = "macos"))]
#[path = "platform/apple.rs"]
pub mod tunnel;

// Tunnel management for Windows
#[cfg(target_os = "windows")]
#[path = "platform/windows.rs"]
pub mod tunnel;

// Tunnel management for Android
#[cfg(target_os = "android")]
#[path = "platform/android.rs"]
pub mod tunnel;
