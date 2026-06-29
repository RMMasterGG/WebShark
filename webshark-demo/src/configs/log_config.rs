use webshark::config::config_traits::Config;
use webshark::config::raw_toml::RawToml;
use webshark::macros::{bean, config, provider};
use webshark::{config, serde};
use webshark::serde::de::DeserializeOwned;
use webshark::serde::Deserialize;

// "prefix" - позволяет пользователю указать группу в toml конфиги (обязательно)


#[config(prefix = "demo_config")]
#[derive(Clone, Default, Deserialize)]
#[serde(crate = "webshark::serde")]
pub struct DemoConfig {
    pub access_time: u32,
    pub refresh_time: u32,
    pub jwt_secret: String,
}


#[provider(settings = "custom")]
impl DemoConfig {

    #[bean]
    pub fn access_and_jwt(cfg: Self) -> Self {

        Self {
            access_time: 0,
            refresh_time: cfg.refresh_time,
            jwt_secret: "".to_string(),
        }
    }
}

// макрос создаёт это (config)
impl Config for DemoConfig {
    fn section_name() -> &'static str {
        "demo"
    }
}

// "config" - читает только из томл, не обращая внимания на код.
// "custom" - читает из томл и после накладывает сверху данные кода.
// "code" - читает только из кода.