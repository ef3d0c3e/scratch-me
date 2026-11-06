use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind: String,
    pub secure_cookies: bool,
    /// Proxies whose X-Forwarded-For is believed
    pub trusted_proxies: Vec<std::net::IpAddr>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Config> {
        let flag = |name: &str| std::env::var(name).is_ok_and(|v| v == "1" || v == "true");
        Ok(Config {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?,
            bind: std::env::var("SCRATCHME_BIND").unwrap_or_else(|_| "127.0.0.1:8081".into()),
            secure_cookies: flag("SCRATCHME_SECURE_COOKIES"),
            trusted_proxies: std::env::var("SCRATCHME_TRUSTED_PROXIES")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().parse().context("SCRATCHME_TRUSTED_PROXIES holds an address that is not an IP"))
                .collect::<anyhow::Result<_>>()?,
        })
    }
}
