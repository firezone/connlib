use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use firezone_tunnel::RTCSessionDescription;
use libs_common::messages::{Id, Interface, Peer, Relay};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct InitGateway {
    pub interface: Interface,
    pub ipv4_masquerade_enabled: bool,
    pub ipv6_masquerade_enabled: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub resources: Vec<Resource>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: Id,
    pub peer: Peer,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionRequest {
    pub user_id: Id,
    pub client: Client,
    pub rtc_sdp: RTCSessionDescription,
    pub relays: Vec<Relay>,
    pub resource: Resource,
}

// rtc_sdp is ignored from eq since RTCSessionDescription doesn't implement this
// this will probably be changed in the future.
impl PartialEq for ConnectionRequest {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
            && self.client == other.client
            && self.relays == other.relays
            && self.resource == other.resource
    }
}

impl Eq for ConnectionRequest {}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum Destination {
    DnsName(String),
    Ip(Vec<IpAddr>),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: Id,
    pub internal_ipv4: Option<Ipv4Addr>,
    pub internal_ipv6: Option<Ipv6Addr>,
    pub resource_address: Destination,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Metrics {
    peers_metrics: Vec<Metric>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Metric {
    pub client_id: Id,
    pub resource_id: Id,
    pub rx_bytes: u32,
    pub tx_bytes: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct RemoveResource {
    pub id: Id,
}

// These messages are the messages that can be recieved
// either by a client or a gateway by the client.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "event", content = "payload")]
// TODO: We will need to re-visit webrtc-rs
#[allow(clippy::large_enum_variant)]
pub enum IngressMessages {
    Init(InitGateway),
    ConnectionRequest(ConnectionRequest),
    AddResource(Resource),
    RemoveResource(RemoveResource),
    UpdateResource(Resource),
}

// These messages can be sent from a gateway
// to a control pane.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case", tag = "event", content = "payload")]
// TODO: We will need to re-visit webrtc-rs
#[allow(clippy::large_enum_variant)]
pub enum EgressMessages {
    ConnectionReady(ConnectionReady),
    Metrics(Metrics),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionReady {
    pub client_id: Id,
    pub gateway_rtc_sdp: RTCSessionDescription,
}

#[cfg(test)]
mod test {
    use libs_common::{control::PhoenixMessage, messages::Interface};

    use super::{IngressMessages, InitGateway};

    #[test]
    fn init_phoenix_message() {
        let m = PhoenixMessage::new(
            "gateway:83d28051-324e-48fe-98ed-19690899b3b6",
            IngressMessages::Init(InitGateway {
                interface: Interface {
                    ipv4: "100.115.164.78".parse().unwrap(),
                    ipv6: "fd00:2011:1111::2c:f6ab".parse().unwrap(),
                    upstream_dns: vec![],
                },
                ipv4_masquerade_enabled: true,
                ipv6_masquerade_enabled: true,
                resources: vec![],
            }),
        );

        let message = r#"{
            "event": "init",
            "payload": {
                "interface": {
                    "ipv4": "100.115.164.78",
                    "ipv6": "fd00:2011:1111::2c:f6ab"
                },
                "ipv4_masquerade_enabled": true, 
                "ipv6_masquerade_enabled": true
            },
            "ref": null,
            "topic": "gateway:83d28051-324e-48fe-98ed-19690899b3b6"
        }"#;
        let ingress_message: PhoenixMessage<IngressMessages, ()> =
            serde_json::from_str(message).unwrap();
        assert_eq!(m, ingress_message);
    }
}
