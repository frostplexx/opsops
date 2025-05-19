use crate::util::op::{get_fields, get_items, get_vaults};
use crate::util::sops_config::{get_sops_config, read_or_create_config, write_config};
use crate::util::sops_structs::{CreationRule, SopsConfig};
use colored::Colorize;
use dialoguer::Confirm;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use serde_yaml::from_str;
use std::io::Read;

pub fn init() {
    match get_sops_config() {
        Some(mut file) => {
            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                eprintln!("{} {}", "❌ Failed to read config file:".red(), e);
                return;
            }

            // Check if onepassworditem field is missing
            if !contents.contains("onepassworditem") {
                println!(
                    "{}",
                    "⚠️  .sops.yaml exists but is missing onepassworditem field.".yellow()
                );
                assign_op_item();
                return;
            }

            let config: SopsConfig = match from_str(&contents) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} {}", "❌ Failed to parse YAML:".red(), e);
                    return;
                }
            };

            // Config file exists with onepassworditem field, do nothing
            println!("{}", "✅ .sops.yaml file exists. No action needed.".green());
        }
        None => {
            println!("{}", "❌ .sops.yaml is missing.".red());

            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Would you like to create a basic .sops.yaml file?")
                .default(true)
                .interact()
                .unwrap()
            {
                // Create a minimal config with creation_rules
                let config = SopsConfig {
                    creation_rules: vec![CreationRule {
                        path_regex: Some(".*".to_string()),
                        age: None,
                        encrypted_regex: None,
                        key_groups: Vec::new(),
                    }],
                    onepassworditem: String::new(),
                };

                if let Err(e) = write_config(&config) {
                    eprintln!("{} {}", "❌ Failed to create config file:".red(), e);
                    return;
                }

                println!("{}", "✅ Created basic .sops.yaml file.".green());
                assign_op_item();
            } else {
                println!("{}", "Please create a .sops.yaml file manually following the guide at: https://github.com/getsops/sops#using-sops-yaml-conf-to-select-kms-pgp-and-age-for-new-files".yellow());
            }
        }
    }
}

fn assign_op_item() {
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to assign an age key from 1Password?")
        .default(true)
        .interact()
        .unwrap()
    {
        // Get the vault names
        let vaults = match get_vaults() {
            Some(vaults) => vaults,
            None => {
                eprintln!("❌ Failed to retrieve vaults.");
                return;
            }
        };
        // If no vaults are found, exit
        if vaults.is_empty() {
            eprintln!("❌ No vaults found.");
            return;
        }
        // Let the user select a vault
        let selected_vault = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a Vault")
            .items(&vaults)
            .interact()
            .unwrap();
        let items = match get_items(&vaults[selected_vault]) {
            Some(vaults) => vaults,
            None => {
                eprintln!("❌ Failed to retrieve items.");
                return;
            }
        };
        // If no vaults are found, exit
        if items.is_empty() {
            eprintln!("❌ No items found.");
            return;
        }
        // Prompt for the 1Password item name
        let selected_item = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an Item")
            .items(&items)
            .interact()
            .unwrap();
        let fields = match get_fields(&items[selected_item], &vaults[selected_vault]) {
            Some(vaults) => vaults,
            None => {
                eprintln!("❌ Failed to retrieve items.");
                return;
            }
        };
        // If no vaults are found, exit
        if fields.is_empty() {
            eprintln!("❌ No items found.");
            return;
        }
        // Prompt for the 1Password item name
        let selected_field = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a Field")
            .items(&fields)
            .interact()
            .unwrap();
        // Handle the chosen vault and op_name further, if necessary
        let reference = format!(
            "op://{}/{}/{}",
            vaults[selected_vault], items[selected_item], fields[selected_field]
        );
        println!("🔐 Writing 1Password reference to config: {}", reference);

        // Read the existing config
        let mut config = match read_or_create_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("❌ Failed to read or create config: {}", e);
                return;
            }
        };

        // Update the config with the new 1Password reference
        config.onepassworditem = reference;

        // Write the updated config back to disk
        if let Err(e) = write_config(&config) {
            eprintln!("❌ Failed to write config: {}", e);
            return;
        }

        println!(
            "{}",
            "✅ Successfully updated .sops.yaml with 1Password reference.".green()
        );
    }
}
