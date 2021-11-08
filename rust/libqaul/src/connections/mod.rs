// Copyright (c) 2021 Open Community Project Association https://ocpa.ch
// This software is published under the AGPLv3 license.

//! # Qaul Connections Modules
//! 
//! The modules define how and where to connect to network interfaces.

pub mod events;
pub mod lan;
pub mod internet;

use libp2p::{
    Multiaddr,
    noise::{Keypair, X25519Spec},
};
use prost::Message;
use serde::{Serialize, Deserialize};

use crate::storage::configuration::Configuration;
use crate::node::Node;
use crate::rpc::Rpc;
use lan::Lan;
use internet::Internet;

/// Import protobuf message definition generated by 
/// the rust module prost-build.
pub mod proto { include!("qaul.rpc.connections.rs"); }

/// enum with all connection modules
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ConnectionModule {
    /// This is a local user and does not need
    /// any further routing.
    Local,
    /// Lan module, for all kind of lan connections,
    /// neighbour nodes are found over mdns.
    Lan,
    /// Connect statically to remote nodes.
    Internet,
    /// BLE module
    Ble,
    /// no connection module known for this
    None,
}

/// Collection of all connections of libqaul
/// each collection is a libp2p swarm
pub struct Connections {
    pub lan: Option<Lan>,
    pub internet: Option<Internet>,
}

impl Connections {
    /// initialize connections
    pub async fn init() -> Connections  {
        // create transport encryption keys for noise protocol
        let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(Node::get_keys())
        .expect("can create auth keys");

        // initialize Lan module
        let lan = Lan::init(auth_keys.clone()).await;

        // initialize Internet overlay module
        let internet = Internet::init(auth_keys).await;

        let conn = Connections{ lan: Some(lan), internet: Some(internet) };

        conn
    }

    /// Initialize connections for android
    /// This is here for debugging reasons
    pub async fn init_android() -> Connections  {
        log::info!("init_android() start");


        // create transport encryption keys for noise protocol
        let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(Node::get_keys())
        .expect("can create auth keys");

        log::info!("init_android() auth_keys generated");


        // initialize Lan module
        let _lan = Lan::init(auth_keys.clone()).await;

        log::info!("init_android() lan initialized");

        // initialize Internet overlay module
        let internet = Internet::init(auth_keys).await;

        log::info!("init_android() internet initialized");

        //let conn = Connections{ lan: None, internet: Some(internet) };
        let conn = Connections{ lan: None, internet: Some(internet) };

        conn
    }

    /// Process incoming RPC request messages
    pub fn rpc(data: Vec<u8>, internet_opt: Option<&mut Internet>) {
        match proto::Connections::decode(&data[..]) {
            Ok(connections) => {
                match connections.message {
                    Some(proto::connections::Message::InternetNodesRequest(_internet_nodes_request)) => {
                        Self::rpc_send_node_list(proto::Info::Request);
                    },
                    Some(proto::connections::Message::InternetNodesAdd(nodes_entry)) => {
                        // check if we have a valid address
                        let mut valid = false;
                        let mut info = proto::Info::AddSuccess;

                        {
                            // get config
                            let mut config = Configuration::get_mut();

                            // add the node to config if the address is valid
                            let address_result: Result<Multiaddr, libp2p::multiaddr::Error> = nodes_entry.address.clone().parse();
                            match address_result {
                                Ok(address) => {
                                    valid = true;

                                    // add to config
                                    config.internet.peers.push(nodes_entry.address);

                                    // connect to node
                                    if let Some(internet) = internet_opt {
                                        Internet::peer_dial(address, &mut internet.swarm);
                                    }
                                },
                                Err(e) => {
                                    log::error!("Not a valid address: {:?}", e);
                                    info = proto::Info::AddErrorInvalid;
                                },
                            }
                        }

                        // save configuration
                        if valid {
                            Configuration::save();
                        }

                        // send response message
                        Self::rpc_send_node_list(info);
                    },
                    Some(proto::connections::Message::InternetNodesRemove(nodes_entry)) => {
                        let mut info = proto::Info::RemoveErrorNotFound;

                        {
                            let mut nodes: Vec<String> = Vec::new();

                            // get config
                            let mut config = Configuration::get_mut();

                            // loop through addresses and remove the equal
                            for addr_string in &config.internet.peers {
                                let string = String::from(addr_string);
                                if string == nodes_entry.address {
                                    // address has been found and is
                                    // therefore removed.
                                    info = proto::Info::RemoveSuccess;
                                } else {
                                    // addresses do not match.
                                    // add this address to the new vector.
                                    nodes.push(string);
                                }
                            }

                            // add new nodes list to configuration
                            config.internet.peers = nodes;
                        }

                        // save configuration
                        Configuration::save();

                        // TODO: stop connection to removed host

                        // send response
                        Self::rpc_send_node_list(info);
                    },
                    _ => {},
                }
            },
            Err(error) => {
                log::error!("{:?}", error);
            },
        }
    }

    /// create and send a node list message
    fn rpc_send_node_list(info: proto::Info) {
        let mut nodes: Vec<proto::InternetNodesEntry> = Vec::new();

        // get list of peer nodes from config
        let config = Configuration::get();

        // fill all the nodes
        for addr_str in &config.internet.peers {
            nodes.push(proto::InternetNodesEntry {
                address: String::from(addr_str),
            });
        }

        // create the protobuf message
        let proto_message = proto::Connections {
            message: Some(proto::connections::Message::InternetNodesList (
                proto::InternetNodesList {
                    info: info as i32,
                    nodes,
                }
            )),
        };

        // send the message
        Self::rpc_send_message(proto_message);
    }

    /// encode and send connections RPC message to UI
    fn rpc_send_message (message: proto::Connections) {
        // encode message
        let mut buf = Vec::with_capacity(message.encoded_len());
        message.encode(&mut buf).expect("Vec<u8> provides capacity as needed");

        // send message
        Rpc::send_message(buf, super::rpc::proto::Modules::Connections.into(), "".to_string(), Vec::new());
    }
}

