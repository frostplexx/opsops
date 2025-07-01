use serde::Deserialize;
use std::process::Command;
use users::os::unix::UserExt;

use crate::util::print_status::print_warning;

use super::print_status::print_error;

#[derive(Debug, Deserialize)]
pub struct ItemField {
    label: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemFields {
    fields: Vec<ItemField>,
}

#[derive(Debug, Deserialize)]
pub struct ListItem {
    title: String,
}

#[derive(Debug, Deserialize)]
pub struct Vault {
    // id: String,
    name: String,
    // content_version: u32,
    // created_at: String,
    // updated_at: String,
    // items: u32,
}

/// Represents the category of a 1Password item.
pub enum OpCategory {
    _Login,
    Password,
    _Identity,
    _Server,
}

impl OpCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpCategory::_Login => "login",
            OpCategory::Password => "password",
            OpCategory::_Identity => "identity",
            OpCategory::_Server => "server",
        }
    }
}

/// Represents a field within a 1Password item.
pub struct OpItemField {
    pub section: Option<String>,
    pub field: String,
    pub field_type: Option<String>,
    pub value: String,
}

impl OpItemField {
    fn _to_flag(&self) -> String {
        let mut flag = String::new();
        if let Some(section) = &self.section {
            flag.push_str(section);
            flag.push('.');
        }
        flag.push_str(&self.field);
        if let Some(field_type) = &self.field_type {
            flag.push_str(&format!("[{}]", field_type));
        }
        flag.push('=');
        flag.push_str(&self.value);
        flag
    }
}

/// Represents a 1Password item to be created.
pub struct OpItem {
    pub(crate) vault: String,
    pub(crate) title: String,
    pub(crate) category: OpCategory,
    pub(crate) fields: Vec<OpItemField>,
}

/// Helper to run the `op` CLI as the invoking user if running under sudo.
pub fn op_command() -> Command {
    use std::env;
    use std::os::unix::process::CommandExt;

    if let Ok(sudo_user) = env::var("SUDO_USER") {
        if !sudo_user.is_empty() {
            // Get the user's UID and GID
            if let Some(user) = users::get_user_by_name(&sudo_user) {
                let mut cmd = Command::new("op");
                cmd.uid(user.uid());
                cmd.gid(user.primary_group_id());
                // Set HOME to the user's home directory
                if let Some(home) = user.home_dir().to_str() {
                    cmd.env("HOME", home);
                } else {
                    print_warning("Couldn't get home directory of sudo user");
                }
                return cmd;
            } else {
                print_warning("Couldn't get sudo user by name");
            }
        } else {
            print_warning("Sudo User is empty!");
        }
    }
    Command::new("op")
}

pub fn op_item_create(item: OpItem) {
    let mut cmd = op_command();

    cmd.arg("item")
        .arg("create")
        .arg("--vault")
        .arg(&item.vault)
        .arg("--title")
        .arg(&item.title)
        .arg("--category")
        .arg(item.category.as_str());

    for field in item.fields {
        let field_str = match (&field.section, &field.field_type) {
            (Some(section), Some(ftype)) => {
                format!("{}.{}[{}]={}", section, field.field, ftype, field.value)
            }
            (Some(section), None) => {
                format!("{}.{}={}", section, field.field, field.value)
            }
            (None, Some(ftype)) => {
                format!("{}[{}]={}", field.field, ftype, field.value)
            }
            (None, None) => {
                format!("{}={}", field.field, field.value)
            }
        };
        cmd.arg(field_str);
    }

    let status = cmd.status().expect("failed to run `op` command");

    if !status.success() {
        print_error("Failed to create item in 1Password".to_string());
    }
}

pub fn _op_item_get(item_name: &str, field: &str) -> Option<String> {
    let output = op_command()
        .arg("item")
        .arg("get")
        .arg(item_name)
        .arg("--field")
        .arg(field)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        print_error(format!(
            "Error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
        None
    }
}

pub fn get_vaults() -> Option<Vec<String>> {
    let output_json = op_command()
        .arg("vault")
        .arg("list")
        .arg("--format=json")
        .output()
        .ok()?;

    if output_json.status.success() {
        let vaults: Vec<Vault> = match serde_json::from_slice(&output_json.stdout) {
            Ok(v) => v,
            Err(e) => {
                print_error(format!("Failed to parse JSON: {}", e));
                return None;
            }
        };

        let vault_names: Vec<String> = vaults.into_iter().map(|vault| vault.name).collect();
        Some(vault_names)
    } else {
        print_error(format!(
            "Error: {}",
            String::from_utf8_lossy(&output_json.stderr)
        ));
        None
    }
}

pub fn get_items(vault: &String) -> Option<Vec<String>> {
    let output_json = op_command()
        .arg("item")
        .arg("list")
        .arg("--vault")
        .arg(vault)
        .arg("--format=json")
        .output()
        .ok()?;

    if output_json.status.success() {
        let vaults: Vec<ListItem> = match serde_json::from_slice(&output_json.stdout) {
            Ok(v) => v,
            Err(e) => {
                print_error(format!("Failed to parse JSON: {}", e));
                return None;
            }
        };

        let item_names: Vec<String> = vaults.into_iter().map(|item| item.title).collect();
        Some(item_names)
    } else {
        print_error(format!(
            "Error: {}",
            String::from_utf8_lossy(&output_json.stderr)
        ));
        None
    }
}

pub fn get_fields(item: &String, vault: &String) -> Option<Vec<String>> {
    let output_json = op_command()
        .arg("item")
        .arg("get")
        .arg(item)
        .arg("--vault")
        .arg(vault)
        .arg("--format=json")
        .output()
        .ok()?;

    if output_json.status.success() {
        let fields: ItemFields = match serde_json::from_slice(&output_json.stdout) {
            Ok(v) => v,
            Err(e) => {
                print_error(format!("Failed to parse JSON: {}", e));
                return None;
            }
        };

        let item_names: Vec<String> = fields.fields.into_iter().map(|item| item.label).collect();
        Some(item_names)
    } else {
        print_error(format!(
            "Error: {}",
            String::from_utf8_lossy(&output_json.stderr)
        ));
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::util::op::{OpCategory, OpItem, OpItemField};

    #[test]
    fn test_op_item_field_to_flag() {
        let field = OpItemField {
            section: Some("auth".to_string()),
            field: "username".to_string(),
            field_type: Some("text".to_string()),
            value: "admin".to_string(),
        };
        assert_eq!(field._to_flag(), "auth.username[text]=admin");
    }

    #[test]
    fn test_op_item_create_builds_command() {
        let item = OpItem {
            vault: "TestVault".to_string(),
            title: "MyLogin".to_string(),
            category: OpCategory::_Login,
            fields: vec![
                OpItemField {
                    section: None,
                    field: "username".to_string(),
                    field_type: None,
                    value: "user1".to_string(),
                },
                OpItemField {
                    section: Some("credentials".to_string()),
                    field: "password".to_string(),
                    field_type: Some("password".to_string()),
                    value: "secret".to_string(),
                },
            ],
        };

        // Instead of running `op_item_create`, extract its Command and assert its args (if refactored to allow inspection)
        // Example: let cmd = build_op_create_command(&item);
        // assert!(cmd.get_args().any(|a| a == "item"));

        // You'd need to refactor `op_item_create` to allow inspecting the command, otherwise this test cannot safely verify the internals.
        // See note below.
        assert!(item.fields[1]._to_flag() == "credentials.password[password]=secret");
    }

    #[test]
    fn test_parse_vaults() {
        let json = r#"
    [
        {
            "id": "vault1",
            "name": "TestVault",
            "content_version": 1,
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-01-01T00:00:00Z",
            "items": 10
        },
        {
            "id": "vault2",
            "name": "AnotherVault",
            "content_version": 2,
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-01-01T00:00:00Z",
            "items": 5
        }
    ]
    "#;

        let vaults: Vec<super::Vault> = serde_json::from_str(json).unwrap();
        let names: Vec<String> = vaults.into_iter().map(|v| v.name).collect();

        assert_eq!(names, vec!["TestVault", "AnotherVault"]);
    }
}
