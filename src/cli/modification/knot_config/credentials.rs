use anyhow::Result;
use inquire::{Confirm, CustomType, Password, Select, Text};
use std::{fmt::Debug, path::PathBuf};
use tracing::debug;

use crate::{
    cli::{autocomplete::path::FilePathCompleter, modification::knot_config::prompt_knot_type},
    knot::{
        KnotType,
        credentials::{AuthMethod, KnotCredentials, SavedAuthMethod},
    },
};

#[derive(Debug, Default)]
pub struct ParsedCreds {
    scheme: Option<String>,
    username: Option<String>,
    host: Option<String>,
    port: Option<u16>,
}

pub fn connection_string(input: &str) -> ParsedCreds {
    let mut creds = ParsedCreds::default();
    let mut s = input.trim();

    if s.is_empty() {
        return creds;
    }

    if let Some(pos) = s.find("://") {
        creds.scheme = Some(s[..pos].to_string());
        s = &s[pos + 3..];
    }

    if let Some(pos) = s.find('@') {
        let user = &s[..pos];
        if !user.is_empty() {
            creds.username = Some(user.to_string());
        }
        s = &s[pos + 1..];
    }

    if let Some(pos) = s.rfind(':') {
        let host_part = &s[..pos];
        let port_part = &s[pos + 1..];

        if !host_part.is_empty() {
            creds.host = Some(host_part.to_string());
        }
        if let Ok(p) = port_part.parse::<u16>() {
            creds.port = Some(p);
        }
    } else if !s.is_empty() {
        creds.host = Some(s.to_string());
    }

    creds
}

impl KnotCredentials {
    pub fn option_username(mut self, username: Option<String>) -> Self {
        if let Some(u) = username {
            self.username = u;
        }
        self
    }

    pub fn option_host(mut self, host: Option<String>) -> Self {
        if let Some(h) = host {
            self.host = h;
        }
        self
    }

    pub fn option_port(mut self, port: Option<u16>) -> Self {
        if let Some(p) = port {
            self.port = p;
        }
        self
    }
}

pub fn prompt_username() -> Result<String> {
    Text::new("Username:").prompt().map_err(Into::into)
}

pub fn prompt_host() -> Result<String> {
    Text::new("Host/IP:").prompt().map_err(Into::into)
}
pub fn prompt_password() -> Result<String> {
    Password::new("Password").prompt().map_err(Into::into)
}

pub fn prompt_port() -> Result<u16> {
    CustomType::<u16>::new("Enter port number:")
        .with_help_message("Must be a valid port (0-65535)")
        .with_default(22)
        .with_error_message("Invalid port number. Please enter a number between 0 and 65535.")
        .prompt()
        .map_err(Into::into)
}

pub fn prompt_connection_limit() -> Result<usize> {
    CustomType::<usize>::new("Number of supported connection sessions:")
        .with_help_message("Must be a valid non-negative number")
        .with_default(1)
        .with_error_message("Invalid number.")
        .prompt()
        .map_err(Into::into)
}

pub fn prompt_auth() -> Result<(AuthMethod, SavedAuthMethod)> {
    let auth_choice = Select::new(
        "Select Authentication Method:",
        vec!["Password", "Private Key", "None"],
    )
    .prompt()?;

    match auth_choice {
        "Password" => Ok((
            AuthMethod::Password("password :P".to_string()),
            SavedAuthMethod::Password,
        )),
        "Private Key" => {
            let key_str = Text::new("Private Key path:")
                .with_placeholder("~/.ssh/id_rsa")
                .with_autocomplete(FilePathCompleter::new(true))
                .prompt()?;

            let use_cert = Confirm::new("Do you want to use OpenSSL Certificate?")
                .with_default(false)
                .prompt()?;

            let cert = if use_cert {
                Some(PathBuf::from(
                    Text::new("OpenSSL Certificate path:")
                        .with_autocomplete(FilePathCompleter::new(true))
                        .prompt()?,
                ))
            } else {
                None
            };

            let key_path = PathBuf::from(key_str);
            Ok((
                AuthMethod::PrivateKey {
                    key_path: key_path.clone(),
                    cert_path: cert.clone(),
                },
                SavedAuthMethod::PrivateKey {
                    key_path,
                    cert_path: cert,
                },
            ))
        }
        _ => Ok((AuthMethod::None, SavedAuthMethod::None)),
    }
}

/// Will set the .auth but also the configuration authentication for saving
/// It is save to deserialize the structure without leaking any sensitive data
pub fn prompt_knot_credentials(
    ktype: Option<&KnotType>,
) -> Result<(Option<KnotCredentials>, KnotType)> {
    if let Some(ktype) = ktype
        && *ktype == KnotType::Local
    {
        return Ok((None, ktype.clone()));
    }

    let raw_input = Text::new("Enter connection string or Host/IP:")
        .with_placeholder("ssh://user@host:22 or user@host:port")
        .with_help_message(
            "You can pass full URLs or partial info; missing parts will be prompted.",
        )
        .prompt()?;

    let cred = if !raw_input.is_empty() {
        connection_string(&raw_input)
    } else {
        ParsedCreds {
            scheme: None,
            username: Some("user".to_string()),
            host: Some("host".to_string()),
            port: Some(22),
        }
    };

    let knot_type = if let Some(ktype) = ktype {
        debug!("Knot type was already set {ktype:?}");
        ktype.clone()
    } else if let Some(ref scheme) = cred.scheme {
        match scheme.as_str() {
            "ssh" => KnotType::SSH,
            "sftp" => KnotType::SFTP,
            "local" => KnotType::Local,
            _ => prompt_knot_type()?,
        }
    } else {
        prompt_knot_type()?
    };

    if knot_type == KnotType::Local {
        return Ok((None, knot_type));
    }

    let mut builder = KnotCredentials::new();
    builder.username = if let Some(username) = cred.username {
        username
    } else {
        prompt_username()?
    };

    builder.host = if let Some(host) = cred.host {
        host
    } else {
        prompt_host()?
    };

    builder.port = match cred.port {
        Some(p) => p,
        None => prompt_port()?,
    };

    builder.connection_limit = prompt_connection_limit()?;

    let (auth, config_auth) = prompt_auth()?;
    builder.auth = auth;
    builder.config_auth = config_auth;

    Ok((Some(builder), knot_type))
}
