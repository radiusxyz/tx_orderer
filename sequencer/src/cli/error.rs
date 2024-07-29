primitives::build_error_kind!(
    pub enum ConfigError {
        // `config_path.rs`
        RemoveConfigDirectory = "Failed to remove the previous configuration directory",
        CreateConfigDirectory = "Failed to create a new configuration directory",
        CreateConfigFile = "Failed to create a new config file",
        CreatePrivateKeyFile = "Failed to create a private key file",
        LoadConfigOption = "Failed to load a config file",
        ParseTomlString = "Failed to parse String to TOML String",
    }
);
