use crate::{GlobalContext, util::sops_config::read_or_create_config};
use age::{
    secrecy::{ExposeSecret, SecretString},
    x25519::Identity,
};
use colored::Colorize;
use std::{process::Command, str::FromStr};

use super::print_status::print_error;

/// Retrieves the Age key from 1Password using the reference stored in .sops.yaml or from command line
/// Returns the key as a string if successful, or an error message if not
pub fn get_age_key_from_1password(context: &GlobalContext) -> Result<String, String> {
    let op_reference = if let Some(opitem) = &context.opitem {
        // Use the opitem from command line
        opitem.clone()
    } else {
        // Read the SOPS config to get the 1Password reference
        let config = read_or_create_config(context)
            .map_err(|e| format!("Failed to read SOPS config: {}", e))?;

        // Check if onepassworditem is set
        if config.onepassworditem.is_empty() {
            return Err(
                "No 1Password reference found in .sops.yaml and none provided via --opitem. Run 'opsops init' to configure."
                    .to_string(),
            );
        }

        config.onepassworditem
    };

    // print_info(format!(
    //     "{} {}",
    //     "🔑 Retrieving Age key from".dimmed(),
    //     op_reference.dimmed()
    // ));

    // Run the op command to get the key
    // Format: op://<vault>/<item>/<field>
    let output = Command::new("op")
        .arg("read")
        .arg(&op_reference)
        .output()
        .map_err(|e| format!("Failed to execute 1Password CLI: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("1Password CLI returned an error: {}", error));
    }

    // Get the output as a string
    let key = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Validate that we got a proper Age key
    if !key.starts_with("AGE-SECRET-KEY-") {
        return Err(
            "Retrieved value is not a valid Age key. It should start with 'AGE-SECRET-KEY-'."
                .to_string(),
        );
    }

    Ok(key)
}

// Extract the public key from the age private key
pub fn extract_public_key(private_key: &str) -> Result<String, &'static str> {
    // Parse the private key into an Identity
    let secret_key = SecretString::from(private_key);
    let identity = match Identity::from_str(secret_key.expose_secret()) {
        Ok(id) => id,
        Err(err) => {
            print_error(format!("{} {}", "Invalid private key format:".red(), err));
            return Err(err);
        }
    };

    // Derive the public key from the private key
    let recipient = identity.to_public();
    let derived_public_key = recipient.to_string();

    Ok(derived_public_key)
}

#[cfg(test)]
mod tests {

    use crate::util::op_key::extract_public_key;

    #[test]
    fn test_extract_public_key_valid() {
        let private_key =
            "AGE-SECRET-KEY-1X9Q72KQG3J383K5SA030D46Q8WTYPDEKV6UA0RXZCXN56YVN22YQMNNCXJ";
        let result = extract_public_key(private_key);
        assert!(result.is_ok());

        let pub_key = result.unwrap();
        assert!(pub_key.starts_with("age1"));
    }

    #[test]
    fn test_extract_public_key_invalid() {
        let invalid_key = "not-a-valid-key";
        let result = extract_public_key(invalid_key);
        assert!(result.is_err());
    }
}
