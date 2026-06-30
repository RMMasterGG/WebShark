use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use serde::Serialize;
use crate::config::config_traits::Config;
use crate::config::raw_toml::RawToml;

#[derive(Clone, serde::Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
        }
    }
}

impl ServerConfig {
    pub fn address(mut self, addr: IpAddr) -> Self {
        self.address = addr;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn server_and_port(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

// Реализуем наш контракт конфигурации
impl Config for ServerConfig {
    fn section_name() -> &'static str {
        "server" // Ищет секцию [server] в TOML
    }

    fn provide(toml: &RawToml) -> Self {
        // Если в TOML файле нет секции [server], берем эталонный дефолт, чтобы не падать
        toml.parse_section::<Self>().unwrap_or_default()
    }
}
