use std::sync::Arc;
use parking_lot::RwLock;

pub const CLEAN_ANYCAST_IPS: &[&str] = &[
    "151.101.1.140",
    "151.101.65.140",
    "151.101.129.140",
    "151.101.193.140",
    "199.232.193.140",
    "199.232.197.140"
];

pub struct FastlyH2Tunnel {
    pub edge_host: String,
    pub edge_port: u16,
    pub active_token: Arc<RwLock<String>>,
}

impl FastlyH2Tunnel {
    pub fn new(host: String, port: u16, token: String) -> Self {
        Self {
            edge_host: host,
            edge_port: port,
            active_token: Arc::new(RwLock::new(token)),
        }
    }

    pub fn swap_token_in_flight(&self, new_token: String) {
        let mut guard = self.active_token.write();
        *guard = new_token;
    }

    pub fn current_token(&self) -> String {
        self.active_token.read().clone()
    }
}
