#[macro_use]
extern crate log;
extern crate android_logger;
extern crate jni;
use self::jni::JNIEnv;
use android_logger::Config;
use firezone_connlib::Session;
use jni::objects::{JClass, JObject, JString, JValue};
use log::LevelFilter;

/// This should be called once after the library is loaded by the system.
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_firezone_connlib_Logger_init(_: JNIEnv, _: JClass) {
    #[cfg(debug_assertions)]
    let level = LevelFilter::Trace;
    #[cfg(not(debug_assertions))]
    let level = LevelFilter::Warn;
    
    android_logger::init_once(
        Config::default()
            // Allow all log levels
            .with_max_level(level)
            .with_tag("connlib"),
    )
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_firezone_connlib_Session_connect(
    mut env: JNIEnv,
    _class: JClass,
    portal_url: JString,
    portal_token: JString,
    callback: JObject,
) -> *const Session {
    let portal_url: String = env.get_string(&portal_url).unwrap().into();
    let portal_token: String = env.get_string(&portal_token).unwrap().into();

    let session = Session::connect(portal_url, portal_token).expect("Failed to connect to portal");

    // TODO: Get actual values from portal
    let tunnelAddressesJSON = "[{\"tunnel_ipv4\": \"100.100.1.1\", \"tunnel_ipv6\": \"fd00:0222:2011:1111:6def:1001:fe67:0012\"}]";
    let tj = env.new_string(tunnelAddressesJSON).unwrap();
    let resourcesJSON = "[{\"id\": \"342b8565-5de2-4289-877c-751d924518e9\", \"label\": \"GitLab\", \"address\": \"gitlab.com\", \"tunnel_ipv4\": \"100.71.55.101\", \"tunnel_ipv6\": \"fd00:0222:2011:1111:6def:1001:fe67:0012\"}]";
    let _rj = env.new_string(resourcesJSON).unwrap();

    // TODO: Get actual IPs returned from portal based on this device
    match env.call_method(
        callback,
        "setTunnelAddresses",
        "(Ljava/lang/String;)Z",
        &[JValue::from(&tj)],
    ) {
        Ok(res) => trace!("setTunnelAddresses returned {:?}", res),
        Err(e) => error!("Failed to call setTunnelAddresses: {:?}", e),
    }

    let session_ptr = Box::into_raw(Box::new(session));

    session_ptr
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_firezone_connlib_Session_disconnect(
    _env: JNIEnv,
    _: JClass,
    session_ptr: *mut Session,
) -> bool {
    if session_ptr.is_null() {
        return false;
    }

    unsafe { Box::from_raw(session_ptr).disconnect() }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_firezone_connlib_Session_bump_sockets(
    session_ptr: *const Session,
) -> bool {
    if session_ptr.is_null() {
        return false;
    }

    unsafe { (*session_ptr).bump_sockets() };

    // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#LL197C6-L197C50
    return true;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_firezone_connlib_disable_some_roaming_for_broken_mobile_semantics(
    session_ptr: *const Session,
) -> bool {
    if session_ptr.is_null() {
        return false;
    }

    unsafe { (*session_ptr).disable_some_roaming_for_broken_mobile_semantics() };

    // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#LL197C6-L197C50
    return true;
}
