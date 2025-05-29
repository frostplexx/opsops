use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use super::{
    print_status::print_error,
    sops_structs::{CreationRule, SopsConfig},
};
use crate::{GlobalContext, util};
use colored::Colorize;
use serde::Deserialize;
use serde_yaml::{from_str, to_string};

pub fn get_sops_config(context: &GlobalContext) -> Option<File> {
    let config_path = if let Some(sops_file_path) = &context.sops_file {
        // Use the explicitly provided path
        PathBuf::from(sops_file_path)
    } else {
        // Use the default behavior - look for .sops.yaml in project root
        let file_name = ".sops.yaml";

        if let Some(project_root) = util::find_project_root::find_project_root() {
            project_root.join(file_name)
        } else {
            print_error(format!(
                "{}",
                "Could not determine project root.".red().bold()
            ));
            return None;
        }
    };

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

    None
}

pub fn read_or_create_config(context: &GlobalContext) -> Result<SopsConfig, String> {
    match get_sops_config(context) {
        Some(mut file) => {
            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                return Err(format!("Failed to read config file: {}", e));
            }

            // Try parsing as-is first
            match from_str::<SopsConfig>(&contents) {
                Ok(mut config) => {
                    // Override onepassworditem if provided via command line
                    if let Some(opitem) = &context.opitem {
                        config.onepassworditem = opitem.clone();
                    }
                    Ok(config)
                }
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
                                // Create a complete config with the parsed rules and onepassworditem from context or empty
                                let onepassworditem = context.opitem.clone().unwrap_or_default();
                                Ok(SopsConfig {
                                    creation_rules: partial.creation_rules,
                                    onepassworditem,
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
            let onepassworditem = context.opitem.clone().unwrap_or_default();
            Ok(SopsConfig {
                creation_rules: Vec::new(),
                onepassworditem,
            })
        }
    }
}

pub fn write_config(config: &SopsConfig, context: &GlobalContext) -> Result<(), String> {
    let config_path = if let Some(sops_file_path) = &context.sops_file {
        // Use the explicitly provided path
        PathBuf::from(sops_file_path)
    } else {
        // Use the default behavior - write to .sops.yaml in project root
        if let Some(project_root) = util::find_project_root::find_project_root() {
            project_root.join(".sops.yaml")
        } else {
            return Err("Could not determine project root".to_string());
        }
    };

    let yaml = match to_string(config) {
        Ok(y) => y,
        Err(e) => return Err(format!("Failed to serialize config: {}", e)),
    };

    let mut file = match File::create(&config_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Failed to create config file {}: {}",
                config_path.display(),
                e
            ));
        }
    };

    if let Err(e) = file.write_all(yaml.as_bytes()) {
        return Err(format!("Failed to write to config file: {}", e));
    }

    Ok(())
}
