// Copyright (c) 2023 Open Community Project Association https://ocpa.ch
// This software is published under the AGPLv3 license.

//! Qaul Community Router
//!
//! This module implements all the tables and logic of the
//! qaul router.

use prost::Message;
use state::InitCell;
use std::sync::RwLock;

pub mod connections;
pub mod feed_requester;
pub mod flooder;
pub mod info;
pub mod neighbours;
pub mod table;
pub mod user_requester;
pub mod users;

use crate::storage::configuration::{Configuration, RoutingOptions};
use connections::ConnectionTable;
use feed_requester::{FeedRequester, FeedResponser};
use flooder::Flooder;
use info::RouterInfo;
use neighbours::Neighbours;
use table::RoutingTable;
use user_requester::{UserRequester, UserResponser};
use users::Users;

/// Import protobuf message definition generated by
/// the rust module prost-build.
pub mod proto {
    include!("qaul.rpc.router.rs");
}
pub mod router_net_proto {
    include!("qaul.net.router_net_info.rs");
}

/// mutable state of router,
/// used for storing the router configuration
static ROUTER: InitCell<RwLock<Router>> = InitCell::new();

/// qaul community router access
#[derive(Clone)]
pub struct Router {
    pub configuration: RoutingOptions,
}

impl Router {
    /// Initialize the qaul router
    pub fn init() {
        let config = Configuration::get();
        let router = Router {
            configuration: config.routing.clone(),
        };
        // set configuration to state
        ROUTER.set(RwLock::new(router));

        // initialize direct neighbours table
        Neighbours::init();

        // initialize users table
        Users::init();

        // initialize flooder queue
        Flooder::init();

        // initialize feed_requester queue
        FeedRequester::init();

        // initialize feed_response queue
        FeedResponser::init();

        // initialize user_requester queue
        UserRequester::init();

        // initialize user_response queue
        UserResponser::init();

        // initialize the global routing table
        RoutingTable::init();

        // initialize the routing information collection
        // tables per connection module
        ConnectionTable::init();

        // initialize RouterInfo submodule that
        // schedules the sending of the routing information
        // to the neighbouring nodes.
        RouterInfo::init(config.routing.sending_table_period);
    }

    /// Get router configuration from state
    pub fn get_configuration() -> RoutingOptions {
        let router = ROUTER.get().read().unwrap();
        router.configuration.clone()
    }

    /// Process incoming RPC request messages and send them to
    /// the submodules
    pub fn rpc(data: Vec<u8>) {
        match proto::Router::decode(&data[..]) {
            Ok(router) => {
                match router.message {
                    Some(proto::router::Message::RoutingTableRequest(_request)) => {
                        // send routing table list
                        RoutingTable::rpc_send_routing_table();
                    }
                    Some(proto::router::Message::ConnectionsRequest(_request)) => {
                        // send connections list
                        ConnectionTable::rpc_send_connections_list();
                    }
                    Some(proto::router::Message::NeighboursRequest(_request)) => {
                        // send neighbours list
                        Neighbours::rpc_send_neighbours_list();
                    }
                    _ => {}
                }
            }
            Err(error) => {
                log::error!("{:?}", error);
            }
        }
    }
}
