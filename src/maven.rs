use std::{
    error::Error,
    io::{BufReader, Read},
};

use serde::{
    Deserialize,
    de::{self, Unexpected},
};

use crate::artifact::{Artifact, TreeParser};

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
struct Dependency {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub version: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub scope: String,
    pub classifier: String,
    #[serde(deserialize_with = "bool_from_string")]
    pub optional: bool,
    #[serde(default)]
    pub children: Vec<Dependency>,
}

fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(de::Error::invalid_value(
            Unexpected::Str(other),
            &"true of false",
        )),
    }
}

impl Into<Artifact> for Dependency {
    fn into(self) -> Artifact {
        let children: Vec<Artifact> = self.children.into_iter().map(|d| d.into()).collect();
        Artifact {
            group_id: self.group_id,
            artifact_id: self.artifact_id,
            version: self.version,
            scope: self.scope,
            children: children,
            latest_version: None,
        }
    }
}

pub struct MavenTreeParser {}

impl TreeParser for MavenTreeParser {
    fn parse(&self, reader: &mut dyn Read) -> Result<Artifact, Box<dyn Error>> {
        let buffered = BufReader::new(reader);
        let root: Dependency = serde_json::from_reader(buffered)?;
        Ok(root.into())
    }
}
