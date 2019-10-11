use identity::Identity;
use crate::{User, ContactBook, auth::AuthStore};

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// Primary context structure for `libqaul`
///
/// Handles user state, secret storage, network state,
/// I/O and services. Check `api` for the extended
/// service API
///
/// ## Bootstrapping
///
/// Starting an instance of `libqaul` requires several steps.
/// For one, it needs to be initialised with a valid config
/// for the routing-layer (`RATMAN`). This requires choosing
/// of network backends and client configuration.
///
/// Secondly, `libqaul` by itself does very little, except handle
/// service requests. The service API exposes various workloads
/// available, but the consuming services also need to be configured,
/// externally to `libqaul` and this instance.
///
/// A bootstrapping procedure should thus look as follows:
///
/// 1. RATMAN + netmod initialisation
/// 2. `libqaul` startup (this struct, call `init()`)
/// 3. Initialise services with a `libqaul` instance reference
/// 4. Your application is now ready for use
#[derive(Clone)]
pub struct Qaul {
    pub(crate) users: Arc<Mutex<BTreeMap<Identity, User>>>,

    /// Handles user tokens and pw hashes
    pub(crate) auth: AuthStore,
    
    pub(crate) contacts: Arc<Mutex<BTreeMap<Identity, ContactBook>>>,
}

impl Qaul {
    pub fn start() -> Self {
        Self {
            users: Arc::new(Mutex::new(BTreeMap::new())),
            auth: AuthStore::new(),
            contacts: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}
