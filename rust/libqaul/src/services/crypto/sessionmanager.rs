// Copyright (c) 2023 Open Community Project Association https://ocpa.ch
// This software is published under the AGPLv3 license.

//! # Crypto Session Manager
//!
//! Handling of the crypto sessions, such as session confirmation.
//!
//! The crypto session manager uses the crypto_net protobuf file
//! containing the Cryptoservice messages.
//! All cryptoservice protobuf messages need to be confirmed by
//! the receiver.

use libp2p::PeerId;
use prost::Message;

use crate::node::user_accounts::UserAccount;
use crate::services::messaging;
use crate::utilities::timestamp::Timestamp;

/// Import protobuf crypto service definition generated by
/// the rust module prost-build.
pub mod proto_net {
    include!("qaul.net.crypto.rs");
}

/// Crypto Session Management
#[derive(Clone)]
pub struct CryptoSessionManager {}

impl CryptoSessionManager {
    /// decode and process crypto session protobuf messages
    pub fn process_cryptoservice_container(
        sender_id: &PeerId,
        user_account: UserAccount,
        data: Vec<u8>,
    ) {
        log::trace!("process_cryptoservice_container");
        // decode protobuf cryptoservice message container
        match proto_net::CryptoserviceContainer::decode(&data[..]) {
            Ok(cryptoservice_container) => match cryptoservice_container.message {
                Some(proto_net::cryptoservice_container::Message::SecondHandshake(
                    second_handshake,
                )) => {
                    Self::process_second_handshake(&user_account, sender_id, second_handshake);
                }
                None => {
                    log::error!(
                        "Cryptoservice message from {} was empty",
                        sender_id.to_base58()
                    )
                }
            },
            Err(e) => {
                log::error!(
                    "Error decoding Cryptoservice Message from {} to {}: {}",
                    sender_id.to_base58(),
                    user_account.id.to_base58(),
                    e
                );
            }
        }
    }

    /// process second hand shake
    fn process_second_handshake(
        user_account: &UserAccount,
        sender_id: &PeerId,
        second_handshake: proto_net::SecondHandshake,
    ) {
        log::trace!("process_second_handshake");

        // confirm reception of the message
        messaging::Messaging::on_confirmed_message(
            &second_handshake.signature,
            sender_id.to_owned(),
            user_account.to_owned(),
            messaging::proto::Confirmation {
                signature: second_handshake.signature.clone(),
                received_at: second_handshake.received_at,
            },
        );
    }

    /// create second handshake protobuf message
    ///
    /// return binary messaging message
    pub fn create_second_handshake_message(signature: Vec<u8>) -> Vec<u8> {
        // create timestamp
        let received_at = Timestamp::get_timestamp();

        // pack message
        let proto_cryptoservice_message = proto_net::CryptoserviceContainer {
            message: Some(
                proto_net::cryptoservice_container::Message::SecondHandshake(
                    proto_net::SecondHandshake {
                        signature,
                        received_at,
                    },
                ),
            ),
        };

        // encode binary message
        let mut cryptoservice_buf = Vec::with_capacity(proto_cryptoservice_message.encoded_len());
        proto_cryptoservice_message
            .encode(&mut cryptoservice_buf)
            .expect("Vec<u8> provides capacity as needed");

        // create messaging message
        let proto_messaging_message = messaging::proto::Messaging {
            message: Some(messaging::proto::messaging::Message::CryptoService(
                messaging::proto::CryptoService {
                    content: cryptoservice_buf,
                },
            )),
        };

        // encode messaging message
        let mut messaging_buf = Vec::with_capacity(proto_messaging_message.encoded_len());
        proto_messaging_message
            .encode(&mut messaging_buf)
            .expect("Vec<u8> provides capacity as needed");

        messaging_buf
    }
}
