use std::{
    error::Error,
    io::{BufReader, Read},
};

use serde::{
    Deserialize,
    de::{self, Unexpected},
};

use crate::artifact::{Artifact, ArtifactParser};

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

impl From<Dependency> for Artifact {
    fn from(value: Dependency) -> Self {
        let children: Vec<Artifact> = value.children.into_iter().map(|d| Self::from(d)).collect();
        Artifact {
            group_id: value.group_id,
            artifact_id: value.artifact_id,
            version: value.version,
            scope: value.scope,
            children: children,
            ..Artifact::default()
        }
    }
}

pub struct MavenArtifactParser<'a> {
    reader: BufReader<&'a mut dyn Read>,
}

impl<'a> MavenArtifactParser<'a> {
    pub fn new(reader: &'a mut dyn Read) -> MavenArtifactParser<'a> {
        MavenArtifactParser {
            reader: BufReader::new(reader),
        }
    }
}

impl ArtifactParser for MavenArtifactParser<'_> {
    fn parse(&mut self) -> Result<Artifact, Box<dyn Error>> {
        let root: Dependency = serde_json::from_reader(&mut self.reader)?;
        Ok(Artifact::from(root))
    }
}
