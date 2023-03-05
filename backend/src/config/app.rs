use crate::config::get_env;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tracing::warn;

const HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 3001;
const ORIGIN: &str = "http://127.0.0.1";

#[derive(Deserialize)]
pub struct ApplicationSettingsModel {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub origin: Option<String>,
}

impl ApplicationSettingsModel {
    pub fn to_settings(self) -> ApplicationSettings {
        let host = self.host.map_or_else(
            || {
                warn!("Using default host");
                HOST
            },
            |host| Ipv4Addr::from_str(&host).expect("Incorrect host"),
        );
        let port = self.port.unwrap_or_else(|| {
            warn!("Using default port");
            PORT
        });

        let addr = SocketAddr::new(IpAddr::V4(host), port);

        ApplicationSettings::new(addr, self.origin.unwrap_or(ORIGIN.to_string()))
    }
}
#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub addr: SocketAddr,
    pub origin: String,
}

impl ApplicationSettings {
    pub fn new(addr: SocketAddr, origin: String) -> Self {
        Self { addr, origin }
    }

    pub fn from_env() -> Self {
        let host = Ipv4Addr::new(0, 0, 0, 0);
        let port = get_env("PORT").parse::<u16>().expect("Invalid port number");
        Self {
            addr: SocketAddr::new(IpAddr::V4(host), port),
            origin: get_env("WEBSITE_URL"),
        }
    }
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(HOST), PORT),
            origin: "http://127.0.0.1".to_string(),
        }
    }
}
