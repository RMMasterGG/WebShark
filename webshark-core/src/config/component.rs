use http::Extensions;
use crate::config::config_traits::Config;
use crate::config::raw_toml::RawToml;

pub struct ConfigComponent {
    file_name: String,
    loaders: Vec<Box<dyn FnOnce(&str, &mut Extensions)>>,
}

impl ConfigComponent {
    pub fn new(file_name: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
            loaders: Vec::new(),
        }
    }

    pub fn add_provider<T>(mut self) -> Self
    where
        T: Config + Clone + 'static + Default,
    {
        let cfg_name = T::section_name();
        self.loaders.push(Box::new(move |full_path, extensions| {
            let toml_storage = RawToml::from_file(full_path);
            let ready_config = T::provide(&toml_storage);
            extensions.insert(ready_config);
        }));

        self
    }

    pub fn loaders(self) -> Vec<Box<dyn FnOnce(&str, &mut Extensions)>> {
        self.loaders
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}