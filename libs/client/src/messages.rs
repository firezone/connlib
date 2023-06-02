use firezone_tunnel::RTCSessionDescription;
use serde::{Deserialize, Serialize};

use libs_common::messages::{Id, Interface, Key, RequestConnection, ResourceDescription};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct InitClient {
    pub interface: Interface,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub resources: Vec<ResourceDescription>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct RemoveResource {
    pub id: Id,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Connect {
    pub rtc_sdp: RTCSessionDescription,
    pub resource_id: Id,
    pub gateway_public_key: Key,
}

// Just because RTCSessionDescription doesn't implement partialeq
impl PartialEq for Connect {
    fn eq(&self, other: &Self) -> bool {
        self.resource_id == other.resource_id && self.gateway_public_key == other.gateway_public_key
    }
}

impl Eq for Connect {}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Relays {
    pub resource_id: Id,
    pub relays: Vec<String>,
}

// These messages are the messages that can be recieved
// by a client.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "event", content = "payload")]
// TODO: We will need to re-visit webrtc-rs
#[allow(clippy::large_enum_variant)]
pub enum IngressMessages {
    Init(InitClient),
    Relays(Relays),
    Connect(Connect),

    // Resources: arrive in an orderly fashion
    AddResource(ResourceDescription),
    RemoveResource(RemoveResource),
    UpdateResource(ResourceDescription),
}

// These messages can be sent from a client to a control pane
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
// TODO: We will need to re-visit webrtc-rs
#[allow(clippy::large_enum_variant)]
pub enum EgressMessages {
    GetConnectionDetails(Id),
    RequestConnection(RequestConnection),
}

#[cfg(test)]
mod test {
    use libs_common::{
        control::PhoenixMessage,
        messages::{Interface, ResourceDescription},
    };

    use super::{IngressMessages, InitClient};

    #[test]
    fn init_phoenix_message() {
        let m = PhoenixMessage::new(
            "device",
            IngressMessages::Init(InitClient {
                interface: Interface {
                    ipv4: "100.76.112.111".parse().unwrap(),
                    ipv6: "fd00:2011:1111::13:efb9".parse().unwrap(),
                    upstream_dns: vec![],
                },
                resources: vec![
                    ResourceDescription {
                        id: "030c2869-6e0d-4dc1-a186-5f1962a1a02b".parse().unwrap(),
                        address: Some("172.172.0.1/16".to_string()),
                        ipv4: "100.69.89.84".parse().unwrap(),
                        ipv6: "fd00:2011:1111::1f:5317".parse().unwrap(),
                    },
                    ResourceDescription {
                        id: "a25fce02-de8e-48e0-b664-287623cfa85e".parse().unwrap(),
                        address: Some("gitlab.mycorp.com".to_string()),
                        ipv4: "100.72.207.207".parse().unwrap(),
                        ipv6: "fd00:2011:1111::1b:3120".parse().unwrap(),
                    },
                ],
            }),
        );
        println!("{}", serde_json::to_string(&m).unwrap());
        let message = r#"
            {
                "event": "init",
                "payload": {
                    "interface": {
                        "ipv4": "100.76.112.111",
                        "ipv6": "fd00:2011:1111::13:efb9",
                        "upstream_dns": []
                    },
                    "resources": [ 
                        {"address": "172.172.0.1/16", "id": "030c2869-6e0d-4dc1-a186-5f1962a1a02b", "ipv4": "100.69.89.84", "ipv6": "fd00:2011:1111::1f:5317"},
                        {"address": "gitlab.mycorp.com", "id": "a25fce02-de8e-48e0-b664-287623cfa85e", "ipv4": "100.72.207.207", "ipv6": "fd00:2011:1111::1b:3120"}
                    ]
                },
                "ref":null,
                "topic": "device"
            }
        "#;
        let ingress_message: PhoenixMessage<IngressMessages> =
            serde_json::from_str(message).unwrap();
        assert_eq!(m, ingress_message);
    }
}
