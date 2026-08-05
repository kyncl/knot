use anyhow::Result;
use keyring::Entry;
use russh::keys::ssh_key::sha2::{Digest, Sha256};
use std::fmt::Write;
use tracing::debug;

use crate::{
    knot::credentials::{AuthMethod, KnotCredentials},
    utils::crypto::{decrypt_password, encrypt_password},
};

fn get_service_name(credentials: &KnotCredentials) -> String {
    let hash = Sha256::digest(format!(
        "{}@{}:{}",
        credentials.username, credentials.host, credentials.port
    ));

    let mut service_name = String::with_capacity(69);
    service_name.push_str("KNOT-");
    for byte in hash {
        write!(&mut service_name, "{:02x}", byte).unwrap();
    }
    debug!("Getting service name: {service_name}");
    service_name
}

pub fn get_password(credentials: &KnotCredentials) -> Result<String> {
    let service = get_service_name(credentials);
    let user = &credentials.username;
    let entry = Entry::new(&service, user)?;
    decrypt_password(&entry.get_password()?)
}

pub fn save_password_new(credentials: &KnotCredentials, password: &str) -> Result<()> {
    let service = get_service_name(credentials);
    let user = &credentials.username;
    let entry = Entry::new(&service, user)?;
    entry.set_password(&encrypt_password(password)?)?;
    entry.get_password()?;
    Ok(())
}

pub fn save_password(credentials: &KnotCredentials) -> Result<()> {
    if let AuthMethod::Password(password) = &credentials.auth {
        save_password_new(credentials, password)?;
    }
    Ok(())
}

pub fn delete_password(credentials: &KnotCredentials) -> Result<()> {
    let service = get_service_name(credentials);
    let user = &credentials.username;
    let entry = Entry::new(&service, user)?;
    entry.set_password("")?;
    entry.delete_credential()?;
    Ok(())
}
