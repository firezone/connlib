use firezone_connlib::Session;

#[swift_bridge::bridge]
mod ffi {
    #[swift_bridge(swift_repr = "struct")]
    struct ResourceList {
        resources: String,
    }

    #[swift_bridge(swift_repr = "struct")]
    struct TunnelAddresses {
        address4: String,
        address6: String,
    }

    extern "Rust" {
        fn connect(portal_url: String, token: String) -> Session;

        #[swift_bridge(swift_name = "bumpSockets")]
        fn bump_sockets(_: &Session) -> bool;

        #[swift_bridge(swift_name = "disableSomeRoamingForBrokenMobileSemantics")]
        fn disable_some_roaming_for_broken_mobile_semantics(_: &Session) -> bool;

        fn disconnect(_: &Session) -> bool;
    }

    extern "Swift" {
        #[swift_bridge(swift_name = "updateResources")]
        fn update_resources(resourceList: ResourceList) -> bool;

        #[swift_bridge(swift_name = "setTunnelAddresses")]
        fn set_tunnel_addresses(tunnelAddresses: TunnelAddresses) -> bool;
    }
}

fn connect(portal_url: String, token: String) -> Session {
    let session = Session::connect(portal_url, token);

    let resources = "[]".to_string();
    ffi::update_resources(ffi::ResourceList { resources });

    // TODO: Get actual IPs returned from portal based on this device
    ffi::set_tunnel_addresses(ffi::TunnelAddresses {
        address4: "100.100.1.1".to_string(),
        address6: "fd00:0222:2021:1111:0000:0000:0001:0002".to_string(),
    });

    session
}

fn bump_sockets(_: &Session) -> bool {
    // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#L177
    return true;
}

fn disable_some_roaming_for_broken_mobile_semantics(_: &Session) -> bool {
    // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#LL197C6-L197C50
    return true;
}

fn disconnect(session: &Session) -> bool {
    session.disconnect()
}
