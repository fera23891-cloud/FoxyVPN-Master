use std::collections::HashSet;

pub struct DnsSinkhole {
    blocked_domains: HashSet<String>,
}

impl DnsSinkhole {
    pub fn new() -> Self {
        let mut blocked = HashSet::new();
        let tracker_list = [
            "doubleclick.net",
            "google-analytics.com",
            "facebook.net",
            "adnxs.com",
            "telemetry.mozilla.org",
            "app-measurement.com",
            "adservice.google.com"
        ];
        for d in tracker_list {
            blocked.insert(d.to_string());
        }
        Self { blocked_domains: blocked }
    }

    pub fn should_block(&self, domain: &str) -> bool {
        let clean = domain.trim().trim_end_matches('.').to_lowercase();
        self.blocked_domains.contains(&clean)
    }
}
