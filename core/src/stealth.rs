pub struct FirefoxFingerprintProfile;

impl FirefoxFingerprintProfile {
    pub const USER_AGENT: &'static str = "MozillaVPN/2.35.0 (sys:linux; iap:true)";
    pub const TLS_CIPHERS: &'static [&'static str] = &[
        "TLS_AES_128_GCM_SHA256",
        "TLS_CHACHA20_POLY1305_SHA256",
        "TLS_AES_256_GCM_SHA384",
        "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    ];
}
