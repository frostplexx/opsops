use age::{secrecy::ExposeSecret, x25519};
use colored::Colorize;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};

use crate::util::{
    op::{OpCategory, OpItem, OpItemField, op_item_create},
    print_status::print_info,
};

pub fn generate_age_key() {
    let key = x25519::Identity::generate();
    let pubkey = key.to_public();

    let label_width = 17;

    println!(
        "{} {}",
        format!("{:width$}", "🔑 Public Key:", width = label_width)
            .yellow()
            .bold(),
        pubkey.to_string().cyan()
    );

    println!(
        "{} {}",
        format!("{:width$}", "🔐 Private Key:", width = label_width)
            .red()
            .bold(),
        key.to_string().expose_secret()
    );

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to save this key in 1Password?")
        .default(false)
        .interact()
        .unwrap()
    {
        let name = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a name for the 1Password item")
            .interact_text()
            .unwrap();
        save_to_op(&key, name);
    } else {
        println!(
            "{}",
            "Remember to save this key in a secure location!".dimmed()
        );
    }

    print_info(format!(
        "{}",
        "Don't forget to add the public key to .sops.yaml!".bold(),
    ));
}

fn save_to_op(key: &x25519::Identity, item_name: String) {
    let item = OpItem {
        vault: "Personal".to_string(),
        title: item_name.to_string(),
        category: OpCategory::Password,
        fields: vec![
            OpItemField {
                section: None,
                field: "Public Key".to_string(),
                field_type: Some("STRING".to_string()),
                value: key.to_public().to_string(),
            },
            OpItemField {
                section: None,
                field: "Private Key".to_string(),
                field_type: Some("PASSWORD".to_string()),
                value: key.to_string().expose_secret().to_string(),
            },
        ],
    };

    op_item_create(item);
}
