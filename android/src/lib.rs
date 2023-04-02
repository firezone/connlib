extern crate jni;
use self::jni::JNIEnv;
use firezone_connlib::Session;
use jni::objects::{JClass, JString};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_firezone_connlib_Session_connect(
    mut env: JNIEnv,
    _: JClass,
    portal_url: JString,
    portal_token: JString,
) -> *const Session {
    let portal_url: String = env.get_string(&portal_url).unwrap().into();
    let portal_token: String = env.get_string(&portal_token).unwrap().into();

    let session = Session::connect(portal_url, portal_token).expect("Failed to connect to portal");

    let resources = "[]".to_string();

    let _resourcesJSON = "[{\"id\": \"342b8565-5de2-4289-877c-751d924518e9\", \"label\": \"GitLab\", \"address\": \"gitlab.com\", \"tunnel_ipv4\": \"100.71.55.101\", \"tunnel_ipv6\": \"fd00:0222:2011:1111:6def:1001:fe67:0012\"}]";
    env.call_static_method(
        "dev/firezone/connlib/Session",
        "updateResources",
        "(Ljava/lang/String;)V",
        &[], // TODO: pass resourceJSON
    )
    .expect("Failed to call updateResources");

    // TODO: Get actual IPs returned from portal based on this device
    let _tunnelAddressesJSON = "[{\"tunnel_ipv4\": \"100.100.1.1\", \"tunnel_ipv6\": \"fd00:0222:2011:1111:6def:1001:fe67:0012\"}]";
    env.call_static_method(
        "dev/firezone/connlib/Session",
        "setTunnelAddresses",
        "(Ljava/lang/String;)V",
        &[], // TODO: pass tunnelAddressesJSON
    )
    .expect("Failed to call setTunnelAddresses");

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
    // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#L177
    return true;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_dev_firezone_connlib_disable_some_roaming_for_broken_mobile_semantics(
    session_ptr: *const Session,
) -> bool {
    // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#LL197C6-L197C50
    return true;
}
