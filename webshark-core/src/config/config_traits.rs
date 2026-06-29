use crate::config::raw_toml::RawToml;
use serde::de::DeserializeOwned;

pub trait Config: DeserializeOwned + Send + Sync + 'static {
    fn section_name() -> &'static str;
    fn file_name() -> Option<&'static str> {
        None
    }

    fn provide(toml: &RawToml) -> Self
    where
        Self: Default,
    {
        Self::toml_mode(toml)
    }

    fn toml_mode(toml: &RawToml) -> Self
    where
        Self: Default,
    {
        toml.parse_section::<Self>().unwrap_or_default()
    }

    // fn custom_mode(toml: &RawToml) -> Self {
    //     let toml_cfg = Self::toml_mode(toml);
    //     Self::apply_code_logic(toml_cfg)
    // }
    //
    // fn code_mode() -> Self where Self: Default {
    //     let default_cfg = Self::default();
    //     Self::apply_code_logic(default_cfg)
    // }
}
