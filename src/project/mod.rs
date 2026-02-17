mod libraries;
mod modules;
mod project_info;
pub use libraries::*;
pub use modules::*;
pub use project_info::*;

use crate::{util::IdPath, Config};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structure of the solar.yaml file
#[derive(Debug, Serialize, Deserialize)]
pub struct SolarConfig {
    pub name: String,
    pub publisher: Option<String>,
    pub version: String,
    pub description: Option<String>,

    author: Option<String>,
    authors: Option<Vec<String>>,

    dependencies: Option<HashMap<String, String>>,
}

impl SolarConfig {
    pub fn read(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Self = serde_yaml::from_str(&content)?;

        Ok(cfg)
    }

    /*
        pub fn basepath(&self) -> IdPath {
            let publisher = if let Some(p) = &self.publisher { p } else { "" };

            vec![
                format!("{}({})", self.name, publisher),
                self.version.to_owned(),
            ]
        }

        pub fn authors(&self) -> Vec<String> {
            let mut v = Vec::new();
            if let Some(a) = &self.author {
                v.push(a.to_string());
            }

            if let Some(authors) = &self.authors {
                for a in authors {
                    v.push(a.to_string());
                }
            }

            v
        }
    */

    pub fn deps(&self) -> Vec<Dependency> {
        let Some(deps) = &self.dependencies else {
            return Vec::new();
        };

        deps.iter()
            .map(|(key, value)| Dependency::from_key_value(key, value).expect("read dependencies"))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub repo: Option<String>,
}

impl Dependency {
    pub fn basepath(&self) -> IdPath {
        let name = &self.name;
        let publisher = &self.publisher;
        let version = &self.version;

        vec![format!("{name}({publisher})"), version.to_owned()]
    }

    /// Gives the directory of the library within the local filesystem.
    pub fn dir(&self, config: &Config) -> String {
        let mut path = config.solarpath.clone();
        if !path.ends_with('/') {
            path.push('/');
        }

        path.push_str("libraries/");

        path + &self.basepath().join("/")
    }

    fn from_key_value(key: &str, value: &str) -> Result<Self, String> {
        let Some((name, rest)) = key.split_once('(') else {
            return Err(format!(
                "expect dependency keys to adhere to pattern '<name>(<publisher>)', but haven't found '(' in key '{key}'",
            ));
        };

        if !rest.ends_with(')') {
            return Err(format!(
                "expect dependency keys to adhere to pattern '<name>(<publisher>)', but haven't found ')' at the end of key '{key}'"
            ));
        }
        let publisher = rest[..rest.len() - 1].to_string();
        let name = name.to_string();

        // the version may be written after declaration of a git repo.
        let (repo, version) = if let Some((repo, version)) = value.rsplit_once('@') {
            let repo = Some(repo.to_string());
            let version = version.to_string();

            (repo, version)
        } else {
            (None, value.to_string())
        };

        Ok(Self {
            name,
            publisher,
            version,
            repo,
        })
    }
}
