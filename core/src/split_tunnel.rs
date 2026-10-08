pub struct SplitTunnelEngine;

impl SplitTunnelEngine {
    pub fn should_bypass_vpn(host: &str) -> bool {
        let clean = host.to_lowercase();
        clean.ends_with(".ir")
            || clean.contains("shaparak")
            || clean.contains("snapp")
            || clean.contains("divar")
            || clean.contains("aparat")
            || clean.contains("telewebion")
            || clean.contains("tamin.ir")
    }
}
