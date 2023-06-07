use firezone_tunnel::RTCSessionDescription;
use serde::{Deserialize, Serialize};

use libs_common::messages::{Id, Interface, Key, Relay, RequestConnection, ResourceDescription};

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

/// List of relays
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Relays {
    /// Resource id corresponding to the relay
    pub resource_id: Id,
    /// The actual list of relays
    pub relays: Vec<Relay>,
}

// These messages are the messages that can be recieved
// by a client.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "event", content = "payload")]
// TODO: We will need to re-visit webrtc-rs
#[allow(clippy::large_enum_variant)]
pub enum IngressMessages {
    Init(InitClient),
    Connect(Connect),

    // Resources: arrive in an orderly fashion
    AddResource(ResourceDescription),
    RemoveResource(RemoveResource),
    UpdateResource(ResourceDescription),
}

/// The replies that can arrive from the channel by a client
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ReplyMessages {
    Relays(Relays),
}

/// The totality of all messages (might have a macro in the future to derive the other types)
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum Messages {
    Init(InitClient),
    Relays(Relays),
    Connect(Connect),

    // Resources: arrive in an orderly fashion
    AddResource(ResourceDescription),
    RemoveResource(RemoveResource),
    UpdateResource(ResourceDescription),
}

impl From<IngressMessages> for Messages {
    fn from(value: IngressMessages) -> Self {
        match value {
            IngressMessages::Init(m) => Self::Init(m),
            IngressMessages::Connect(m) => Self::Connect(m),
            IngressMessages::AddResource(m) => Self::AddResource(m),
            IngressMessages::RemoveResource(m) => Self::RemoveResource(m),
            IngressMessages::UpdateResource(m) => Self::UpdateResource(m),
        }
    }
}

impl From<ReplyMessages> for Messages {
    fn from(value: ReplyMessages) -> Self {
        match value {
            ReplyMessages::Relays(m) => Self::Relays(m),
        }
    }
}

// These messages can be sent from a client to a control pane
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "event", content = "payload")]
// TODO: We will need to re-visit webrtc-rs
#[allow(clippy::large_enum_variant)]
pub enum EgressMessages {
    ListRelays { resource_id: Id },
    RequestConnection(RequestConnection),
}

#[cfg(test)]
mod test {
    use libs_common::{
        control::PhoenixMessage,
        messages::{Interface, Relay, ResourceDescription, Stun, Turn},
    };

    use crate::messages::{EgressMessages, Relays, ReplyMessages};

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
        let ingress_message: PhoenixMessage<IngressMessages, ReplyMessages> =
            serde_json::from_str(message).unwrap();
        assert_eq!(m, ingress_message);
    }

    #[test]
    fn list_relays_message() {
        let m = PhoenixMessage::<EgressMessages, ()>::new(
            "device",
            EgressMessages::ListRelays {
                resource_id: "f16ecfa0-a94f-4bfd-a2ef-1cc1f2ef3da3".parse().unwrap(),
            },
        );
        let message = r#"
            {
                "event": "list_relays",
                "payload": {
                    "resource_id": "f16ecfa0-a94f-4bfd-a2ef-1cc1f2ef3da3"
                },
                "ref":null,
                "topic": "device"
            }
        "#;
        let egress_message = serde_json::from_str(&message).unwrap();
        assert_eq!(m, egress_message);
    }

    #[test]
    fn list_relays_reply() {
        let m = PhoenixMessage::<IngressMessages, ReplyMessages>::new_reply(
            "device",
            ReplyMessages::Relays(Relays {
                resource_id: "f16ecfa0-a94f-4bfd-a2ef-1cc1f2ef3da3".parse().unwrap(),
                relays: vec![
                    Relay::Stun(Stun {
                        uri: "stun:189.172.73.111:3478".to_string(),
                    }),
                    Relay::Turn(Turn {
                        expires_at: 1686629954,
                        uri: "turn:189.172.73.111:3478".to_string(),
                        username: "1686629954:C7I74wXYFdFugMYM".to_string(),
                        password: "OXXRDJ7lJN1cm+4+2BWgL87CxDrvpVrn5j3fnJHye98".to_string(),
                    }),
                    Relay::Stun(Stun {
                        uri: "stun:::1:3478".to_string(),
                    }),
                    Relay::Turn(Turn {
                        expires_at: 1686629954,
                        uri: "turn:::1:3478".to_string(),
                        username: "1686629954:dpHxHfNfOhxPLfMG".to_string(),
                        password: "8Wtb+3YGxO6ia23JUeSEfZ2yFD6RhGLkbgZwqjebyKY".to_string(),
                    }),
                ],
            }),
        );
        println!("{}", serde_json::to_string(&m).unwrap());
        let message = r#"
            {
                "ref":null,
                "topic":"device",
                "event": "phx_reply",
                "payload": {
                    "response": {
                        "relays": [
                            {
                                "type":"stun",
                                "uri":"stun:189.172.73.111:3478"
                            },
                            {
                                "expires_at": 1686629954,
                                "password": "OXXRDJ7lJN1cm+4+2BWgL87CxDrvpVrn5j3fnJHye98",
                                "type": "turn",
                                "uri": "turn:189.172.73.111:3478",
                                "username":"1686629954:C7I74wXYFdFugMYM"
                            },
                            {
                                "type": "stun",
                                "uri": "stun:::1:3478"
                            },
                            {
                                "expires_at": 1686629954,
                                "password": "8Wtb+3YGxO6ia23JUeSEfZ2yFD6RhGLkbgZwqjebyKY",
                                "type": "turn",
                                "uri": "turn:::1:3478",
                                "username": "1686629954:dpHxHfNfOhxPLfMG"
                            }],
                        "resource_id": "f16ecfa0-a94f-4bfd-a2ef-1cc1f2ef3da3"
                    },
                    "status":"ok"
                }
            }"#;
        let reply_message = serde_json::from_str(&message).unwrap();
        assert_eq!(m, reply_message);
    }
}
