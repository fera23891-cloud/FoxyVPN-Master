use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct AccountCredentials {
    pub id: String,
    pub email: String,
    pub session_token: String,
    pub bearer_token: String,
    pub quota_max_bytes: u64,
    pub quota_used_bytes: u64,
    pub is_exhausted: bool,
}

#[derive(Clone)]
pub struct AccountPool {
    accounts: Arc<RwLock<Vec<AccountCredentials>>>,
    current_index: Arc<AtomicUsize>,
    sticky_domain_map: Arc<RwLock<HashMap<String, usize>>>,
}

impl AccountPool {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(AtomicUsize::new(0)),
            sticky_domain_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_account(&self, acc: AccountCredentials) {
        let mut list = self.accounts.write().await;
        list.push(acc);
    }

    pub async fn get_token_for_domain(&self, domain: &str) -> Option<String> {
        let accounts = self.accounts.read().await;
        if accounts.is_empty() {
            return None;
        }

        // 1. Check sticky session first (keeps bank/sensitive sites consistent)
        let mut sticky = self.sticky_domain_map.write().await;
        if let Some(&idx) = sticky.get(domain) {
            if idx < accounts.len() && !accounts[idx].is_exhausted {
                return Some(accounts[idx].bearer_token.clone());
            }
        }

        // 2. Load balance to next available healthy account
        for (i, acc) in accounts.iter().enumerate() {
            if !acc.is_exhausted {
                sticky.insert(domain.to_string(), i);
                return Some(acc.bearer_token.clone());
            }
        }

        None
    }

    /// Called instantly upon HTTP 429 Quota Exceeded to hot-swap without dropping connection
    pub async fn hot_swap_exhausted_token(&self, exhausted_token: &str) -> Option<String> {
        let mut accounts = self.accounts.write().await;
        for acc in accounts.iter_mut() {
            if acc.bearer_token == exhausted_token {
                acc.is_exhausted = true;
                break;
            }
        }

        accounts.iter()
            .find(|a| !a.is_exhausted)
            .map(|a| a.bearer_token.clone())
    }
}
