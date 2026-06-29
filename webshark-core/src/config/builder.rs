use std::fmt::format;
use bytes::Buf;
use crate::config::component::ConfigComponent;
use crate::config::config_traits::Config;
use crate::config::raw_toml::RawToml;
use http::Extensions;
use crate::config::server_config::ServerConfig;

pub struct SingleFileMode {
    file_path: String,
}

pub struct MultiFileMode {
    base_dir: String,
}

pub struct ConfigBuilder<Mode> {
    mode: Mode,
    registry: Extensions,
}

impl Default for ConfigBuilder<SingleFileMode> {
    fn default() -> Self {
        let mut registry = Extensions::new();
        registry.insert(ServerConfig::default());
        Self {
            mode: SingleFileMode {
                file_path: "./webshark.toml".to_string(),
            },
            registry,
        }
    }
}

impl ConfigBuilder<SingleFileMode> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(mut self, path: &str) -> Self {
        self.mode.file_path = path.to_string();
        self
    }

    pub fn add_provider<T>(mut self) -> Self
    where
        T: Config + Clone + Default,
    {
        let toml_storage = RawToml::from_file(&self.mode.file_path);
        let ready_config = T::provide(&toml_storage);
        self.registry.insert(ready_config);
        self
    }

    pub fn global_dir(self, path: &str) -> ConfigBuilder<MultiFileMode> {
        ConfigBuilder {
            mode: MultiFileMode {
                base_dir: path.to_string(),
            },
            registry: self.registry,
        }
    }
}

impl ConfigBuilder<MultiFileMode> {
    pub fn add_component(mut self, component: ConfigComponent) -> Self {
        let full_path = format!("{}/{}", self.mode.base_dir, component.file_name());
        for loader in component.loaders() {
            loader(&full_path, &mut self.registry);
        }
        self
    }
}

impl<Mode> ConfigBuilder<Mode> {
    pub fn build(self) -> Extensions {
        self.registry
    }
}
