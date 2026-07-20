use anyhow::Result;
use russh::keys::ssh_encoding::bytes::Bytes;
use russh::keys::*;
use russh::*;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use crate::knot::credentials::{AuthMethod, KnotCredentials};

pub mod pool;

// let path_to_private_key: Option<PathBuf> = None;
// let mut ssh = Session::connect(
//     path_to_private_key,
//     inquire::prompt_text("Username").unwrap_or("root".to_string()),
//     Some(inquire::prompt_text("Password").unwrap()),
//     None,
//     (
//         inquire::prompt_text("Host").unwrap(),
//         inquire::prompt_u32("Port").unwrap() as u16,
//     ),
// )
// .await?;
// info!("Connected");
// let command = inquire::prompt_text("Command").unwrap();
// info!("{:?}", command);
// let code = ssh.call(&command).await?;
// println!("Exitcode: {code:?}");
// ssh.close().await?;

pub struct Client {}
impl client::Handler for Client {
    type Error = russh::Error;
    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct Session {
    pub session: client::Handle<Client>,
}

impl Session {
    pub async fn connect(credentials: &KnotCredentials) -> Result<Self> {
        // Load ssh certificate
        let openssh_cert = match &credentials.auth {
            AuthMethod::PrivateKey { cert_path, .. } => cert_path
                .clone()
                .map(load_openssh_certificate)
                .transpose()?,
            _ => None,
        };

        let config = client::Config {
            inactivity_timeout: Some(Duration::from_secs(300)),
            keepalive_interval: Some(Duration::from_secs(15)),
            preferred: Preferred {
                kex: Cow::Owned(vec![
                    russh::kex::CURVE25519_PRE_RFC_8731,
                    russh::kex::EXTENSION_SUPPORT_AS_CLIENT,
                ]),
                ..Default::default()
            },
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};

        let mut session = client::connect(
            config,
            format!("{}:{}", credentials.host, credentials.port),
            sh,
        )
        .await?;
        match &credentials.auth {
            AuthMethod::Password(password) => {
                let auth_res = session
                    .authenticate_password(&credentials.username, password)
                    .await?;

                if !auth_res.success() {
                    anyhow::bail!("Authentication with password failed");
                }
            }
            AuthMethod::PrivateKey { key_path, .. } => {
                let key_pair = load_secret_key(key_path, None)?;
                let arc_key = Arc::new(key_pair);

                let auth_res = if let Some(cert) = openssh_cert {
                    session
                        .authenticate_openssh_cert(&credentials.username, arc_key, cert)
                        .await?
                } else {
                    let hash_alg = session.best_supported_rsa_hash().await?.flatten();
                    session
                        .authenticate_publickey(
                            &credentials.username,
                            PrivateKeyWithHashAlg::new(arc_key, hash_alg),
                        )
                        .await?
                };

                if !auth_res.success() {
                    anyhow::bail!("Authentication with public key / cert failed");
                }
            }
            AuthMethod::None => {
                anyhow::bail!("No authentication method provided");
            }
        }

        Ok(Self { session })
    }

    pub async fn call(&self, command: &str) -> Result<(u32, Option<Bytes>)> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut code = None;
        let mut collected_bytes = Vec::new();

        loop {
            let Some(msg) = channel.wait().await else {
                break;
            };

            match msg {
                ChannelMsg::Data { ref data } => {
                    collected_bytes.extend_from_slice(&data[..]);
                }
                ChannelMsg::ExtendedData { ref data, ext: _ } => {
                    collected_bytes.extend_from_slice(&data[..]);
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }
                ChannelMsg::Eof => {}
                ChannelMsg::Close => {
                    break;
                }
                _ => {}
            }
        }
        let exit_code = code.ok_or_else(|| {
            anyhow::anyhow!(
                "Remote SSH command closed without returning an exit status code. Command: '{}'",
                command
            )
        })?;
        let final_bytes = if collected_bytes.is_empty() {
            None
        } else {
            Some(Bytes::from(collected_bytes))
        };

        Ok((exit_code, final_bytes))
    }

    pub async fn close(&self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}
