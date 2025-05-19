use crate::util::op_key::extract_public_key;
use crate::util::{op_key, sops_config};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::Path;
use std::ffi::OsString;

// Set encryption patterns for a file in .sops.yaml
pub fn set_keys(path: OsString) {
    let path_str = path.to_string_lossy().to_string();
    let file_path = Path::new(&path_str);

    // Check if the file exists
    if !file_path.exists() {
        eprintln!("{} {}", "Error:".red().bold(), "File not found.".red());
        return;
    }

    // Verify the file extension (only YAML and JSON are supported)
    if let Some(ext) = file_path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if !["yaml", "yml", "json"].contains(&ext_str.as_str()) {
            eprintln!(
                "{} {}",
                "Error:".red().bold(),
                "Only YAML and JSON files are supported.".red()
            );
            return;
        }
    } else {
        eprintln!(
            "{} {}\n",
            "Error:".red().bold(),
            "File has no extension. Only YAML and JSON files are supported.".red()
        );
        return;
    }

    // Ensure we have the key from 1Password
    match op_key::get_age_key_from_1password() {
        Ok(key) => {
            // Extract public key from the private key
            let pubkey = match extract_public_key(&key) {
                Ok(k) => k,
                Err(err) => {
                    eprint!("{}{}", "❌ Error getting public key: \n".red(), err);
                    return;
                }
            };
            if pubkey.is_empty() {
                eprintln!(
                    "{} {}",
                    "Error:".red().bold(),
                    "Could not extract public key from the age key.\n".red()
                );
                return;
            }

            // Get the file name for the rule
            let file_name = file_path.to_string_lossy();

            // Prompt the user for encryption options
            let encrypted_regex = match prompt_for_encryption_pattern() {
                Ok(t) => t,
                Err(error) => {
                    eprint!("{}: {}", "❌ Error getting regex\n".red(), error);
                    return;
                }
            };

            // Update the SOPS configuration
            match update_sops_config(&file_name, &pubkey, &encrypted_regex) {
                Ok(_) => {
                    print!("{}", "✅ Successfully updated .sops.yaml\n".green());
                }
                Err(err) => {
                    eprint!("{}: {}", "❌ Error updating .sops.yam\n".red(), err);
                    return;
                }
            }

            println!("You can now encrypt your file with:\n");
            println!("  {} {}\n", "opsops encrypt".yellow(), path_str.yellow());
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e.red());
        }
    }
}

// Prompt the user to choose an encryption pattern
fn prompt_for_encryption_pattern() -> std::io::Result<String> {
    let options = vec![
        "All values (encrypt entire file)",
        "Kubernetes (data, stringData, password, ingress, token fields)",
        "Talos configuration secrets (secrets sections, certs, keys)",
        "Common sensitive data (passwords, tokens, keys, credentials)",
        "Custom pattern (provide your own regex)",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What do you want to encrypt in this file?")
        .default(0)
        .items(&options)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let encrypted_regex = match selection {
        0 => Ok(".*".to_string()),
        1 => Ok("^(data|stringData|password|token|secret|key|cert|ca.crt|tls|ingress|backupTarget)"
            .to_string()),
        2 => Ok("^(secrets|privateKey|token|key|crt|cert|password|secret|kubeconfig|talosconfig)"
            .to_string()),
        3 => Ok("^(password|token|secret|key|auth|credential|private|apiKey|cert)".to_string()),
        4 => {
            dialoguer::Input::<String>::new()
                .with_prompt("Enter your regex pattern to match keys you want to encrypt\nExample: ^(password|api_key|secret)")
                .interact()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
        _ => Ok(".*".to_string()),
    }?;

    Ok(encrypted_regex)
}

// Update the SOPS configuration with the new encryption pattern
fn update_sops_config(file_name: &str, pubkey: &str, encrypted_regex: &str) -> std::io::Result<()> {
    // Read the current SOPS configuration
    let mut config = match sops_config::read_or_create_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!(
                "{} {}",
                "Error:".red().bold(),
                format!("Failed to read SOPS config: {}", e).red()
            );
            return Ok(());
        }
    };

    // Check if there's an existing rule for this file
    let mut existing_rule_index = None;
    for (i, rule) in config.creation_rules.iter().enumerate() {
        if let Some(path_regex) = &rule.path_regex {
            if path_regex == file_name {
                existing_rule_index = Some(i);
                break;
            }
        }
    }

    if let Some(index) = existing_rule_index {
        // Update existing rule
        if let Some(rule) = config.creation_rules.get_mut(index) {
            rule.age = Some(pubkey.to_string());
            rule.encrypted_regex = Some(encrypted_regex.to_string());
        }
    } else {
        // Create a new rule
        let new_rule = crate::util::sops_structs::CreationRule {
            path_regex: Some(file_name.to_string()),
            age: Some(pubkey.to_string()),
            encrypted_regex: Some(encrypted_regex.to_string()),
            key_groups: vec![],
        };

        // Add rule to configuration
        config.creation_rules.push(new_rule);
    }

    // Write the updated configuration
    if let Err(e) = sops_config::write_config(&config) {
        eprintln!(
            "{} {}",
            "Error:".red().bold(),
            format!("Failed to write SOPS config: {}", e).red()
        );
        return Ok(());
    }

    Ok(())
}
