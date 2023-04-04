use firezone_connlib::Session;
use std::sync::Arc;

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
        type WrappedSession;

        #[swift_bridge(associated_to = WrappedSession)]
        fn connect(
            portal_url: String,
            token: String,
            callback_handler: CallbackHandler,
        ) -> WrappedSession;

        #[swift_bridge(swift_name = "bumpSockets")]
        fn bump_sockets(&self) -> bool;

        #[swift_bridge(swift_name = "disableSomeRoamingForBrokenMobileSemantics")]
        fn disable_some_roaming_for_broken_mobile_semantics(&self) -> bool;

        fn disconnect(&self) -> bool;
    }

    extern "Swift" {
        type CallbackHandler;

        #[swift_bridge(swift_name = "onUpdateResources")]
        fn on_update_resources(&self, resourceList: ResourceList) -> bool;

        #[swift_bridge(swift_name = "onSetTunnelAddresses")]
        fn on_set_tunnel_addresses(&self, tunnelAddresses: TunnelAddresses) -> bool;
    }
}

pub struct WrappedSession {
    session: Session,
}

impl WrappedSession {
    fn connect(portal_url: String, token: String, callback_handler: ffi::CallbackHandler) -> Self {
        let session = Session::connect(portal_url, token);

        let resources = "[]".to_string();
        let cb = Arc::new(callback_handler);
        let callback_handler = Arc::clone(&cb);

        callback_handler.on_update_resources(ffi::ResourceList { resources });
        callback_handler.on_set_tunnel_addresses(ffi::TunnelAddresses {
            address4: "100.100.1.1".to_string(),
            address6: "fd00:0222:2021:1111:0000:0000:0001:0002".to_string(),
        });

        WrappedSession {
            session: session.unwrap(),
        }
    }

    fn bump_sockets(&self) -> bool {
        // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#L177
        return true;
    }

    fn disable_some_roaming_for_broken_mobile_semantics(&self) -> bool {
        // TODO: See https://github.com/WireGuard/wireguard-apple/blob/2fec12a6e1f6e3460b6ee483aa00ad29cddadab1/Sources/WireGuardKitGo/api-apple.go#LL197C6-L197C50
        return true;
    }

    fn disconnect(&self) -> bool {
        self.session.disconnect()
    }
}
