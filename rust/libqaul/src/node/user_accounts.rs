// Copyright (c) 2023 Open Community Project Association https://ocpa.ch
// This software is published under the AGPLv3 license.

//! User Account Module
//!
//! In qaul each user is defined by the following things
//!
//! * user ID (hash of the public key)
//! * Public / private key
//! * user name (optional)

use base64::Engine;
use libp2p::{
    identity::{ed25519, Keypair, PublicKey},
    PeerId,
};
use prost::Message;
use state::InitCell;
use std::sync::RwLock;

use crate::router;
use crate::rpc::Rpc;
use crate::storage::configuration;
use crate::storage::configuration::Configuration;

/// Import protobuf message definition generated by
/// the rust module prost-build.
pub mod proto {
    include!("qaul.rpc.user_accounts.rs");
}

/// mutable state of users table
static USERACCOUNTS: InitCell<RwLock<UserAccounts>> = InitCell::new();

#[derive(Clone)]
pub struct UserAccount {
    pub id: PeerId,
    pub keys: Keypair,
    pub name: String,
}

pub struct UserAccounts {
    pub users: Vec<UserAccount>,
}

impl UserAccounts {
    pub fn init() {
        let mut accounts = UserAccounts { users: Vec::new() };

        // check if there are users defined in configuration
        let config = Configuration::get();
        let config_users = config.user_accounts.clone();
        let mut iter = IntoIterator::into_iter(config_users);

        while let Some(user) = iter.next() {
            let mut basedecode = base64::engine::general_purpose::STANDARD
                .decode(&user.keys)
                .unwrap();
            let ed25519_keys = ed25519::Keypair::try_from_bytes(&mut basedecode).unwrap();
            let keys = Keypair::from(ed25519_keys);
            let id = PeerId::from(keys.public());

            // check if saved ID and the id from the keypair are equal
            if id.to_string() == user.id {
                log::trace!("user id's of '{}' match {}", user.name, user.id);
            } else {
                log::error!("------------------------------------");
                log::error!("ERROR: user id's of '{}' are not equal", user.name);
                log::error!("{}  {}", id.to_string(), user.id);
                log::error!("------------------------------------");
            }

            // push to user accounts table
            accounts.users.push(UserAccount {
                name: user.name.clone(),
                id,
                keys: keys.clone(),
            });
        }

        // save users to state
        USERACCOUNTS.set(RwLock::new(accounts));
    }

    /// create a new user account with user name
    pub fn create(name: String) -> UserAccount {
        // create user
        let keys_ed25519 = Keypair::generate_ed25519();
        let keys_config = base64::engine::general_purpose::STANDARD
            .encode(keys_ed25519.clone().try_into_ed25519().unwrap().to_bytes());
        /*
        {
            // questions:
            // - what did we save before? private key? keypair?
            //   - answer: keypair!: ed25519::Keypair

            // get the binary ed25519 keypair
            let ed25519_binary_keypair =
                keys_ed25519.clone().try_into_ed25519().unwrap().to_bytes();

            // test the key encoding
            log::info!("==============================");
            log::info!("Test Key Saveing Configuration");
            log::info!("------------------------------");
            log::info!("keys_config: {}", keys_config.clone());

            // convert the string into a keypair again
            let mut binary2 = base64::engine::general_purpose::STANDARD
                .decode(&keys_config)
                .unwrap();

            // binary comparison of the keypairs
            log::info!("binary keypair before encoding:");
            log::info!("{:?}", ed25519_binary_keypair);
            log::info!("binary keypair after encoding: ");
            log::info!("{:?}", binary2);
            log::info!("------------------------------");

            // reconvert into a Keypair structure
            let ed_kp2 = ed25519::Keypair::try_from_bytes(&mut binary2).unwrap();
            let _kp2 = Keypair::from(ed_kp2);

            log::info!("==============================");
        }
         */
        let id = PeerId::from(keys_ed25519.public());
        let user = UserAccount {
            id,
            keys: keys_ed25519.clone(),
            name: name.clone(),
        };

        // save it to state
        let mut users = USERACCOUNTS.get().write().unwrap();
        users.users.push(user.clone());

        // save it to config
        {
            let mut config = Configuration::get_mut();
            config.user_accounts.push(configuration::UserAccount {
                name: name.clone(),
                id: id.to_string(),
                keys: keys_config,
                storage: configuration::StorageOptions::default(),
            });
        }
        Configuration::save();

        // add it to users list
        crate::router::users::Users::add(id, keys_ed25519.public(), name.clone(), false, false);

        // add user to routing table / connections table
        crate::router::connections::ConnectionTable::add_local_user(id);

        // display id
        log::trace!("created user account '{}' {:?}", name, id);

        user
    }

    /// get user account by id
    pub fn get_by_id(account_id: PeerId) -> Option<UserAccount> {
        // get state
        let accounts = USERACCOUNTS.get().read().unwrap();

        // search for ID in accounts
        let mut account_result = None;
        for item in &accounts.users {
            if item.id == account_id {
                account_result = Some(item.clone());
                break;
            }
        }

        account_result
    }

    /// Return the number of registered user accounts on this node.
    #[allow(dead_code)]
    pub fn len() -> usize {
        let users = USERACCOUNTS.get().read().unwrap();
        users.users.len()
    }

    /// Return the default user.
    /// The first registered user account is returned.
    pub fn get_default_user() -> Option<UserAccount> {
        // get state
        let users = USERACCOUNTS.get().read().unwrap();

        // check if a user exists
        if users.users.len() == 0 {
            return None;
        }

        // get user account
        let user = users.users.first().unwrap();
        // Some(UserAccount {
        //     id: user.id.clone(),
        //     keys: user.keys.clone(),
        //     name: user.name.clone(),
        // });
        Some(user.clone())
    }

    /// to fill the routing table get all users
    pub fn get_user_info() -> Vec<router::users::User> {
        let mut user_info = Vec::new();

        let users = USERACCOUNTS.get().read().unwrap();
        for user in &users.users {
            user_info.push(router::users::User {
                id: user.id,
                key: user.keys.public(),
                name: user.name.clone(),
                verified: false,
                blocked: false,
            });
        }

        user_info
    }

    /// checks if user account exists
    ///
    /// returns true if a user account with the given ID exists
    #[allow(dead_code)]
    pub fn is_account(user_id: PeerId) -> bool {
        // get user accounts state
        let users = USERACCOUNTS.get().read().unwrap();

        // loop through user accounts and compare
        for user in &users.users {
            if user.id == user_id {
                return true;
            }
        }

        false
    }

    /// Process incoming RPC request messages for user accounts
    pub fn rpc(data: Vec<u8>) {
        match proto::UserAccounts::decode(&data[..]) {
            Ok(user_accounts) => {
                match user_accounts.message {
                    Some(proto::user_accounts::Message::GetDefaultUserAccount(_)) => {
                        // create message
                        let proto_message;
                        match Self::get_default_user() {
                            Some(user_account) => {
                                // get RPC key values
                                let (key_type, key_base58) =
                                    Self::get_protobuf_public_key(user_account.keys.public());

                                // pack user into protobuf message
                                proto_message = proto::UserAccounts {
                                    message: Some(
                                        proto::user_accounts::Message::DefaultUserAccount(
                                            proto::DefaultUserAccount {
                                                user_account_exists: true,
                                                my_user_account: Some(proto::MyUserAccount {
                                                    name: user_account.name,
                                                    id: user_account.id.to_bytes(),
                                                    id_base58: user_account.id.to_base58(),
                                                    key: user_account
                                                        .keys
                                                        .public()
                                                        .encode_protobuf(),
                                                    key_type,
                                                    key_base58,
                                                }),
                                            },
                                        ),
                                    ),
                                };
                            }
                            None => {
                                // there is no default user so send this information
                                proto_message = proto::UserAccounts {
                                    message: Some(
                                        proto::user_accounts::Message::DefaultUserAccount(
                                            proto::DefaultUserAccount {
                                                user_account_exists: false,
                                                my_user_account: None,
                                            },
                                        ),
                                    ),
                                };
                            }
                        }

                        // encode message
                        let mut buf = Vec::with_capacity(proto_message.encoded_len());
                        proto_message
                            .encode(&mut buf)
                            .expect("Vec<u8> provides capacity as needed");

                        // send message
                        Rpc::send_message(
                            buf,
                            crate::rpc::proto::Modules::Useraccounts.into(),
                            "".to_string(),
                            Vec::new(),
                        );
                    }
                    Some(proto::user_accounts::Message::CreateUserAccount(create_user_account)) => {
                        // create user account
                        let user_account = Self::create(create_user_account.name);

                        // get RPC key values
                        let (key_type, key_base58) =
                            Self::get_protobuf_public_key(user_account.keys.public());

                        // return new user account
                        let proto_message = proto::UserAccounts {
                            message: Some(proto::user_accounts::Message::MyUserAccount(
                                proto::MyUserAccount {
                                    name: user_account.name,
                                    id: user_account.id.to_bytes(),
                                    id_base58: user_account.id.to_base58(),
                                    key: user_account.keys.public().encode_protobuf(),
                                    key_type,
                                    key_base58,
                                },
                            )),
                        };

                        // encode message
                        let mut buf = Vec::with_capacity(proto_message.encoded_len());
                        proto_message
                            .encode(&mut buf)
                            .expect("Vec<u8> provides capacity as needed");

                        // send message
                        Rpc::send_message(
                            buf,
                            crate::rpc::proto::Modules::Useraccounts.into(),
                            "".to_string(),
                            Vec::new(),
                        );
                    }
                    _ => {}
                }
            }
            Err(error) => {
                log::error!("{:?}", error);
            }
        }
    }

    /// create the qaul RPC definitions of a public key
    ///
    /// Returns a tuple with the key type & the base58 encoded
    /// (key_type: String, key_base58: String)
    fn get_protobuf_public_key(key: PublicKey) -> (String, String) {
        // extract values
        let key_type: String;
        let key_base58: String;

        #[allow(unreachable_patterns)]
        match key.try_into_ed25519() {
            Ok(ed_key) => {
                key_type = "Ed25519".to_owned();
                key_base58 = bs58::encode(ed_key.to_bytes()).into_string();
            }
            _ => {
                key_type = "UNDEFINED".to_owned();
                key_base58 = "UNDEFINED".to_owned();
            }
        }

        (key_type, key_base58)
    }
}
