use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SopsConfig {
    #[serde(default)]
    pub creation_rules: Vec<CreationRule>,
    pub onepassworditem: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreationRule {
    pub path_regex: Option<String>,
    pub age: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_regex: Option<String>,
    #[serde(default)]
    pub key_groups: Vec<KeyGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyGroup {
    #[serde(default)]
    pub age: Vec<String>,
}
