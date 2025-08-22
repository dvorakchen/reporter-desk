// use std::collections::HashMap;

use std::{fs, path::PathBuf};

use config::{Config, File};
use log::debug;

const DEFAULT_CONFIG_FILE: &str = "config";

const ALI_DASHSCOPE_API_KEY_KEY: &str = "ALI_DASHSCOPE_API_KEY";
const OPENAI_KEY_KEY: &str = "OPENAI_KEY";

pub struct GlobalConfig {
    config: Config,
}

impl GlobalConfig {
    pub fn new() -> Self {
        let config = Config::builder()
            .add_source(File::with_name(DEFAULT_CONFIG_FILE))
            .build()
            .unwrap();

        Self { config }
    }

    pub fn get_ali_dashscope_api_key(&self) -> String {
        self.config.get_string(ALI_DASHSCOPE_API_KEY_KEY).unwrap()
    }

    pub fn get_deepseek_api_key(&self) -> String {
        self.config.get_string(OPENAI_KEY_KEY).unwrap()
    }

    pub fn set_deepseek_api_key(&mut self, key: String) {
        debug!("set deepseek api key to: {}", key);
        let ali_key = self.get_ali_dashscope_api_key();

        let content = Self::build_content(key, ali_key);

        let mut file_name = PathBuf::from(DEFAULT_CONFIG_FILE);
        file_name.set_extension("toml");

        fs::write(&file_name, content).expect("set config file failed");

        self.refresh();
    }

    pub fn set_ali_dashscope_api_key(&mut self, key: String) {
        debug!("set ali dashscope api key to: {}", key);

        let deepseek_key = self.get_deepseek_api_key();

        let content = Self::build_content(deepseek_key, key);

        let mut file_name = PathBuf::from(DEFAULT_CONFIG_FILE);
        file_name.set_extension("toml");

        fs::write(&file_name, content).expect("set config file failed");

        self.refresh();
    }

    fn refresh(&mut self) {
        let config = Config::builder()
            .add_source(File::with_name(DEFAULT_CONFIG_FILE))
            .build()
            .unwrap();

        debug!("refresh config");
        debug!("new deepseek api key: {}", self.get_deepseek_api_key());
        debug!("new ali api key: {}", self.get_ali_dashscope_api_key());
        self.config = config;
    }

    fn build_content(deepseek_api_key: String, ali_dashscope_api_key: String) -> String {
        format!(
            r#"OPENAI_KEY="{}"
ALI_DASHSCOPE_API_KEY="{}""#,
            deepseek_api_key, ali_dashscope_api_key
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{DEFAULT_CONFIG_FILE, GlobalConfig};
    use serial_test::serial;
    use std::path::PathBuf;

    #[test]
    #[serial]
    fn find_config_file() {
        let mut file = PathBuf::from(DEFAULT_CONFIG_FILE);
        file.set_extension("toml");

        assert!(file.exists());
    }

    #[test]
    #[serial]
    fn set_deepseek_api_key() {
        let mut file = PathBuf::from(DEFAULT_CONFIG_FILE);
        file.set_extension("toml");

        assert!(file.exists());

        const FAKE_KEY: &str = "FAKE_KEY";

        let mut config = GlobalConfig::new();
        let old_value = config.get_deepseek_api_key();

        config.set_deepseek_api_key(FAKE_KEY.to_owned());
        let new_value = config.get_deepseek_api_key();

        assert_eq!(new_value, FAKE_KEY);
        config.set_deepseek_api_key(old_value.clone());
        let new_value = config.get_deepseek_api_key();
        assert_eq!(new_value, old_value);
    }

    #[test]
    #[serial]
    fn set_ali_api_key() {
        let mut file = PathBuf::from(DEFAULT_CONFIG_FILE);
        file.set_extension("toml");

        assert!(file.exists());

        const FAKE_KEY: &str = "FAKE_KEY";

        let mut config = GlobalConfig::new();
        let old_value = config.get_ali_dashscope_api_key();

        config.set_ali_dashscope_api_key(FAKE_KEY.to_owned());
        let new_value = config.get_ali_dashscope_api_key();

        assert_eq!(new_value, FAKE_KEY);
        config.set_ali_dashscope_api_key(old_value.clone());
        let new_value = config.get_ali_dashscope_api_key();
        assert_eq!(new_value, old_value);
    }
}
