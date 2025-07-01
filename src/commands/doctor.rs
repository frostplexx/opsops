use crate::{
    GlobalContext,
    util::{
        op_key::{extract_public_key, get_age_key_from_1password},
        print_status::{print_error, print_success, print_warning},
        sops_config::read_or_create_config,
    },
};
use colored::Colorize;

pub fn doctor(context: &GlobalContext) {
    match which::which("sops") {
        Ok(path) => {
            let version = std::process::Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|out| out.lines().next().unwrap_or("unknown").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            print_success(format!(
                "{} {} {}",
                "Found sops:".green(),
                path.display(),
                version.trim().dimmed()
            ));
        }
        Err(_) => {
            print_error(format!(
                "{}",
                "sops is not installed or not found in PATH. Please install sops.".red()
            ));
            return;
        }
    }
    match which::which("op") {
        Ok(path) => {
            let version = std::process::Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_else(|| "unknown".to_string());
            print_success(format!(
                "{} {} {}",
                "Found 1Password CLI (op):".green(),
                path.display(),
                version.trim().dimmed()
            ));
        }
        Err(_) => {
            print_error(format!(
                "{}",
                "1Password CLI (op) is not installed or not found in PATH. Please install op."
                    .red()
            ));
            return;
        }
    }

    let config = match read_or_create_config(context) {
        Ok(c) => c,
        Err(err) => {
            print_error(format!("{} {}", "Error reading sops file: ".red(), err));
            return;
        }
    };
    // Check if onepassworditem is set
    if config.onepassworditem.is_empty() {
        print_error(format!(
            "{}",
            "No 1Password reference found in .sops.yaml. Run 'opsops init' to configure.".red()
        ));
        return;
    } else {
        print_success(format!(
            "{} {}\n",
            "1Password item found in .sops.yaml:".green(),
            config.onepassworditem
        ));
    }

    let age = match get_age_key_from_1password(context) {
        Ok(it) => it,
        Err(err) => {
            print_error(format!("{} {}", "Couldn't get age key:".red(), err));
            return;
        }
    };

    // Create a copy of age before moving it
    let age_copy = age.clone();
    let mut hiddenkey = age_copy;
    let stars = "*".repeat(hiddenkey.len() - 22);
    hiddenkey.replace_range(15..=(hiddenkey.len() - 8), &stars);
    print_success(format!("{} {}", "Got private key:".green(), hiddenkey));

    // Parse the private key into an Identity
    let derived_public_key = match extract_public_key(&age) {
        Ok(k) => k,
        Err(err) => {
            print_error(format!("{}{}", "Error getting public key: \n".red(), err));
            return;
        }
    };

    // Get public keys from config
    let mut found = false;
    let mut rules_without_age = Vec::new();

    // Check single keys in creation rules and collect rules without age keys
    for (i, rule) in config.creation_rules.iter().enumerate() {
        let mut rule_has_keys = false;

        // Check direct age key
        if let Some(key) = &rule.age {
            rule_has_keys = true;
            if derived_public_key == *key {
                print_success(format!("{} {}", "Found matching public key:".green(), key));
                found = true;
                break;
            }
        }

        // Check key groups
        for key_group in &rule.key_groups {
            if !key_group.age.is_empty() {
                rule_has_keys = true;
                for key in &key_group.age {
                    if derived_public_key == *key {
                        print_success(format!(
                            "{} {}",
                            "Found matching public key in .sops.yaml:".green(),
                            key
                        ));
                        found = true;
                        break;
                    }
                }
            }
            if found {
                break;
            }
        }

        // If this rule has no age keys at all, record it
        if !rule_has_keys {
            rules_without_age.push(i);
        }

        if found {
            break;
        }
    }

    if !found {
        print_error(format!(
            "{}",
            "No matching public key found in .sops.yaml config.".red()
        ));
        print_warning(format!(
            "{}",
            format!("  Your public key is: {}", derived_public_key).yellow()
        ));

        // Print rules without age keys
        if !rules_without_age.is_empty() {
            print_warning(format!("{}", "  Rules without age keys:".yellow()));
            for i in rules_without_age {
                let path_regex = match &config.creation_rules[i].path_regex {
                    Some(regex) => regex.as_str(),
                    None => "<no path_regex>",
                };
                eprintln!("  - Rule #{}: {}", i, path_regex);
            }
        }
    }
}
