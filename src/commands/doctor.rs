use crate::util::{
    op_key::{extract_public_key, get_age_key_from_1password},
    sops_config::read_or_create_config,
};
use colored::Colorize;

pub fn doctor() {
    let config = match read_or_create_config() {
        Ok(c) => c,
        Err(err) => {
            eprint!("{} {}", "❌ Error reading sops file: ".red(), err);
            return;
        }
    };
    // Check if onepassworditem is set
    if config.onepassworditem.is_empty() {
        eprint!(
            "{}",
            "❌ No 1Password reference found in .sops.yaml. Run 'opsops init' to configure.".red()
        );
        return;
    } else {
        print!(
            "{} {}\n",
            "✅ 1Password reference found in .sops.yaml:".green(),
            config.onepassworditem
        );
    }

    let age = match get_age_key_from_1password() {
        Ok(it) => it,
        Err(err) => {
            eprintln!("{} {}", "❌ Couldn't get age key:".red(), err);
            return;
        }
    };

    // Create a copy of age before moving it
    let age_copy = age.clone();
    let mut hiddenkey = age_copy;
    let stars = "*".repeat(hiddenkey.len() - 22);
    hiddenkey.replace_range(15..=(hiddenkey.len() - 8), &stars);
    print!("{} {}\n", "✅ Got private key:".green(), hiddenkey);

    // Parse the private key into an Identity
    let derived_public_key = match extract_public_key(&age) {
        Ok(k) => k,
        Err(err) => {
            eprint!("{}{}", "❌ Error getting public key: \n".red(), err);
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
                print!("{} {}\n", "✅ Found matching public key:".green(), key);
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
                        print!(
                            "{} {}\n",
                            "✅ Found matching public key in key group:".green(),
                            key
                        );
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
        eprintln!(
            "{}",
            "❌ No matching public key found in .sops.yaml config.".red()
        );
        eprintln!(
            "{}",
            format!("  Your public key is: {}", derived_public_key).yellow()
        );

        // Print rules without age keys
        if !rules_without_age.is_empty() {
            eprintln!("{}", "  Rules without age keys:".yellow());
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
