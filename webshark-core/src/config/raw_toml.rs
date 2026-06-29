use std::fs;
use std::path::Path;
use tracing::warn;
use crate::config::config_traits;
use crate::config::server_config::ServerConfig;

pub struct RawToml {
    inner: config::Config,
}

impl RawToml {
    fn create_file(toml_path: &str) {
        let path = Path::new(toml_path);

        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let default_cfg = ServerConfig::default();

            let mut wrapper = std::collections::HashMap::new();
            wrapper.insert("server", default_cfg);

            match toml::to_string_pretty(&wrapper) {
                Ok(toml_default) => {
                    let file_content = format!("# Конфигурация веб-сервера Webshark\n\n{}", toml_default);

                    if let Err(e) = fs::write(path, file_content) {
                        panic!("[webshark] Не удалось записать дефолтный конфиг: {}", e);
                    } else {
                        println!("[webshark] Создан дефолтный файл конфигурации по пути: {}", toml_path);
                    }
                }
                Err(e) => {
                    panic!("[webshark] Ошибка сериализации дефолтного конфига: {}", e);
                }
            }
        }
    }

    pub fn from_file(toml_path: &str) -> Self {
        Self::create_file(toml_path);

        let config = config::Config::builder()
            .add_source(config::File::with_name(toml_path))
            .add_source(
                config::Environment::with_prefix("WEBSHARK")
                    .separator("__")
            )
            .build()
            .unwrap_or_else(|err| {
                panic!("Критическая ошибка: Не удалось прочитать файл {}: {}", toml_path, err);
            });

        Self {
            inner: config
        }
    }

    pub fn parse_section<T>(&self) -> Result<T, config::ConfigError>
    where
        T: config_traits::Config,
    {
        let section_name = T::section_name();
        self.inner.get::<T>(section_name)
    }
}
