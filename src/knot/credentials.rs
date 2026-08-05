use anyhow::Result;
use inquire::{Confirm, Password};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use strum::Display;
use tracing::warn;

use crate::utils::password::{get_password, save_password_new};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Display)]
#[serde(tag = "type")]
pub enum SavedAuthMethod {
    Password,
    PrivateKey {
        key_path: PathBuf,
        cert_path: Option<PathBuf>,
    },
    #[default]
    None,
}
impl SavedAuthMethod {
    /// Resolves persistent auth settings into runtime credentials
    /// Prompting the user for a password if required
    pub fn to_runtime_auth(&self, credentials: &KnotCredentials) -> Result<AuthMethod> {
        match self {
            SavedAuthMethod::Password => {
                let status = get_password(credentials);
                if let Err(err) = &status {
                    warn!("Couldn't get password. Cause: {err}");
                }
                let pass = if let Ok(password) = status {
                    password
                } else {
                    let msg = format!(" Password for '{}:{}'", credentials.host, credentials.port);
                    let p = Password::new(&msg).prompt()?;
                    if Confirm::new("Do you want to save this password into keyring?")
                        .with_default(false)
                        .prompt()?
                        && let Err(err) = save_password_new(credentials, &p)
                    {
                        eprintln!("Failed to save password. Cause: {err}");
                    }
                    p
                };
                Ok(AuthMethod::Password(pass))
            }
            SavedAuthMethod::PrivateKey {
                key_path,
                cert_path,
            } => Ok(AuthMethod::PrivateKey {
                key_path: key_path.clone(),
                cert_path: cert_path.clone(),
            }),
            SavedAuthMethod::None => Ok(AuthMethod::None),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum AuthMethod {
    Password(String),
    PrivateKey {
        key_path: PathBuf,
        cert_path: Option<PathBuf>,
    },
    // SshAgent,
    #[default]
    None,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct KnotCredentials {
    pub username: String,
    pub host: String,
    pub port: u16,
    #[serde(skip)]
    pub auth: AuthMethod,
    #[serde(rename = "authentication")]
    pub config_auth: SavedAuthMethod,
    /// Number of allowed connections for knot
    /// Why here? Because it's more convenient
    /// With this Knot::new doesn't need new parameter `connection_limit` just
    /// for it to be `1`, which is literally default
    /// It kind of makes sense all other properties here are useless for local knot
    pub connection_limit: usize,
}

impl KnotCredentials {
    pub fn new() -> Self {
        Self {
            port: 22,
            connection_limit: 1,
            ..Default::default()
        }
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets number of connection
    pub fn limit(mut self, limit: usize) -> Self {
        self.connection_limit = limit;
        self
    }

    pub fn auth_password(mut self, password: impl Into<String>) -> Self {
        self.auth = AuthMethod::Password(password.into());
        self
    }

    pub fn auth_private_key(
        mut self,
        key_path: impl AsRef<Path>,
        cert_path: Option<PathBuf>,
    ) -> Self {
        self.auth = AuthMethod::PrivateKey {
            key_path: key_path.as_ref().to_path_buf(),
            cert_path,
        };
        self
    }
}
