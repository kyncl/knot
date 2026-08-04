use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::{fs, path::Path};
use toml_edit::{Item, Table, value};

use crate::knot::credentials::{KnotCredentials, SavedAuthMethod};

/// Generic helper to read and parse a TOML file into any struct
pub fn load_toml_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let path = path.as_ref();
    let data =
        fs::read_to_string(path).with_context(|| format!("Failed to read file at {path:?}"))?;
    toml::from_str(&data).with_context(|| format!("Failed to parse TOML from {path:?}"))
}

/// Helper to safely remove multiple keys from an Item if it is a Table
fn remove_keys(item: &mut Item, keys: &[&str]) {
    if let Some(table) = item.as_table_mut() {
        for key in keys {
            table.remove(key);
        }
    }
}

/// Mutates an Item to insert or update the `[credentials]` block
pub fn apply_credentials_to_item(cred: &KnotCredentials, parent_item: &mut Item) {
    if parent_item.get("credentials").is_none() {
        parent_item["credentials"] = Item::Table(Table::new());
    }

    let cred_item = &mut parent_item["credentials"];
    cred_item["username"] = value(&cred.username);
    cred_item["host"] = value(&cred.host);
    cred_item["port"] = value(cred.port as i64);
    cred_item["connection_limit"] = value(cred.connection_limit as i64);

    if cred_item.get("authentication").is_none() {
        cred_item["authentication"] = Item::Table(Table::new());
    }

    let auth_item = &mut cred_item["authentication"];

    match &cred.config_auth {
        SavedAuthMethod::Password => {
            auth_item["type"] = value("Password");
            remove_keys(auth_item, &["key_path", "cert_path"]);
        }
        SavedAuthMethod::PrivateKey {
            key_path,
            cert_path,
        } => {
            auth_item["type"] = value("PrivateKey");
            auth_item["key_path"] = value(key_path.to_string_lossy().as_ref());

            if let Some(cert) = cert_path {
                auth_item["cert_path"] = value(cert.to_string_lossy().as_ref());
            } else {
                remove_keys(auth_item, &["cert_path"]);
            }
        }
        SavedAuthMethod::None => {
            auth_item["type"] = value("None");
            remove_keys(auth_item, &["key_path", "cert_path"]);
        }
    }
}
