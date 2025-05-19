use std::{
    fs::File,
    io::{Read, Write},
};

use super::{
    print_status::print_error,
    sops_structs::{CreationRule, SopsConfig},
};
use crate::util;
use colored::Colorize;
use serde::Deserialize;
use serde_yaml::{from_str, to_string};

pub fn get_sops_config() -> Option<File> {
    let file_name = ".sops.yaml";

    if let Some(project_root) = util::find_project_root::find_project_root() {
        // println!("Project root: {}", project_root.display());

        let config_path = project_root.join(file_name);
        if config_path.exists() {
            match File::open(&config_path) {
                Ok(file) => return Some(file),
                Err(e) => {
                    print_error(format!("Failed to open {}: {}", config_path.display(), e));
                    return None;
                }
            }
        } else {
            print_error(format!(
                "{} {}",
                "Config file not found:".red().bold(),
                config_path.display()
            ));
        }
    } else {
        print_error(format!(
            "{}",
            "Could not determine project root.".red().bold()
        ));
    }

    None
}

pub fn read_or_create_config() -> Result<SopsConfig, String> {
    match get_sops_config() {
        Some(mut file) => {
            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                return Err(format!("Failed to read config file: {}", e));
            }

            // Try parsing as-is first
            match from_str::<SopsConfig>(&contents) {
                Ok(config) => Ok(config),
                Err(e) => {
                    // If parsing fails due to missing onepassworditem field, parse manually
                    if e.to_string().contains("missing field `onepassworditem`") {
                        // Use a custom approach to parse the config without the onepassworditem field
                        #[derive(Deserialize)]
                        struct PartialConfig {
                            #[serde(default)]
                            creation_rules: Vec<CreationRule>,
                        }

                        // Try to parse the partial config
                        match from_str::<PartialConfig>(&contents) {
                            Ok(partial) => {
                                // Create a complete config with the parsed rules and empty onepassworditem
                                Ok(SopsConfig {
                                    creation_rules: partial.creation_rules,
                                    onepassworditem: String::new(),
                                })
                            }
                            Err(e) => Err(format!("Failed to parse partial YAML config: {}", e)),
                        }
                    } else {
                        Err(format!("Failed to parse YAML: {}", e))
                    }
                }
            }
        }
        None => {
            // Create a new config with default values
            Ok(SopsConfig {
                creation_rules: Vec::new(),
                onepassworditem: String::new(),
            })
        }
    }
}

pub fn write_config(config: &SopsConfig) -> Result<(), String> {
    if let Some(project_root) = util::find_project_root::find_project_root() {
        let config_path = project_root.join(".sops.yaml");
        let yaml = match to_string(config) {
            Ok(y) => y,
            Err(e) => return Err(format!("Failed to serialize config: {}", e)),
        };

        let mut file = match File::create(config_path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Failed to create config file: {}", e)),
        };

        if let Err(e) = file.write_all(yaml.as_bytes()) {
            return Err(format!("Failed to write to config file: {}", e));
        }

        Ok(())
    } else {
        Err("Could not determine project root".to_string())
    }
}
