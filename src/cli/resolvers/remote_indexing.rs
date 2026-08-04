use anyhow::{Result, anyhow};
use colored::*;
use inquire::Select;

use crate::configuration::loader::remote::ProvidesRemoteConfig;

pub fn resolve_remote_index<K: ProvidesRemoteConfig>(
    index: Option<usize>,
    knots: &[K],
) -> Result<usize> {
    if let Some(idx) = index {
        if knots.get(idx).is_some() {
            Ok(idx)
        } else {
            Err(anyhow!("Remote knot index {idx} is out of bounds"))
        }
    } else if !knots.is_empty() {
        if knots.len() == 1 {
            Ok(0)
        } else {
            let options: Vec<String> = knots
                .iter()
                .map(|r| {
                    let r = &r.get_config();
                    let ktype = &r.config.adapter_type;
                    let config = &r.config;
                    let path = config.path.display();
                    let creds = config.credentials.as_ref().map_or_else(String::new, |c| {
                        format!(
                            " ({}//{}@{}:{})",
                            format!("{:?}", ktype).green(),
                            c.username.bold(),
                            c.host.yellow(),
                            c.port
                        )
                        .dimmed()
                        .to_string()
                    });

                    format!("{path}{creds}")
                })
                .collect();
            let ans = Select::new("Pick a remote knot:", options).raw_prompt()?;
            Ok(ans.index)
        }
    } else {
        Err(anyhow!(
            "Couldn't find any remote knots. Please add new remote knot or check your configuration"
        ))
    }
}
