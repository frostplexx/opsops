use colored::*;
use serde_yaml::from_str;
use std::io::Read;

use crate::{
    GlobalContext,
    util::{
        print_status::{print_error, print_info},
        sops_config::get_sops_config,
        sops_structs::SopsConfig,
    },
};

pub fn list_config(context: &GlobalContext) {
    let mut file = match get_sops_config(context) {
        Some(f) => f,
        None => {
            print_error(format!(
                "{}",
                "Error: No SOPS configuration file found.".red()
            ));
            return;
        }
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        print_error(format!("{} {}", "Failed to read config file:".red(), e));
        return;
    }

    let config: SopsConfig = match from_str(&contents) {
        Ok(c) => c,
        Err(e) => {
            print_error(format!("{} {}", "Failed to parse YAML:".red(), e));
            return;
        }
    };

    print_info(format!(
        "{} {}\n",
        "Assigned 1Password item:".cyan(),
        config.onepassworditem.green()
    ));
    print!("{}", "Rules:".cyan());

    for (i, rule) in config.creation_rules.iter().enumerate() {
        println!();
        println!("{} {}", "🔹 Rule #".yellow(), (i + 1).to_string().yellow());

        if let Some(pattern) = &rule.path_regex {
            println!("{} {}", "  📂 File pattern:".cyan(), pattern.green());
        }

        if !rule.key_groups.is_empty() {
            let mut any_age = false;
            for group in &rule.key_groups {
                if !group.age.is_empty() {
                    if !any_age {
                        println!("{}", "  🔑 Age Keys:".cyan());
                        any_age = true;
                    }
                    for key in &group.age {
                        println!("    - {}", key.green());
                    }
                }
            }
        }

        if let Some(age_key) = &rule.age {
            println!("{} {}", "  🔑 Age Key:".cyan(), age_key.green());
        }
    }

    println!();
    print_info(format!(
        "{}",
        "This configuration will be used when encrypting files with SOPS.".dimmed()
    ));
}
