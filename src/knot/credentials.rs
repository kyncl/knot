use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(tag = "type")]
pub enum AuthMethod {
    #[serde(skip)]
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
    pub auth: AuthMethod,
}

impl KnotCredentials {
    pub fn new() -> Self {
        Self {
            port: 22,
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
