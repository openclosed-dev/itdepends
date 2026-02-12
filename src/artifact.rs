use serde::{
    Deserialize,
    de::{self, Unexpected},
};

use std::{
    cmp::Ordering,
    collections::HashSet,
    error::Error,
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter, Read, Write},
};

#[derive(Deserialize, Clone, Debug)]
pub struct Artifact {
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
    pub children: Vec<Artifact>,
    #[serde(skip)]
    pub latest_version: String,
}

impl PartialEq for Artifact {
    fn eq(&self, other: &Self) -> bool {
        self.group_id == other.group_id
            && self.artifact_id == other.artifact_id
            && self.version == other.version
            && self.classifier == other.classifier
    }
}

impl Eq for Artifact {}

impl Ord for Artifact {
    fn cmp(&self, other: &Self) -> Ordering {
        self.group_id
            .cmp(&other.group_id)
            .then(self.artifact_id.cmp(&other.artifact_id))
            .then(self.version.cmp(&other.version))
            .then(self.classifier.cmp(&other.classifier))
    }
}

impl PartialOrd for Artifact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Artifact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.group_id.hash(state);
        self.artifact_id.hash(state);
        self.version.hash(state);
        self.classifier.hash(state);
    }
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

impl Artifact {
    pub fn flatten(self) -> Vec<Artifact> {
        let root_group_id = &self.group_id;
        let mut set: HashSet<Artifact> = HashSet::new();
        add_artifacts_to_set(self.children, &mut set);
        let mut flattened: Vec<Artifact> = set.into_iter().collect();
        flattened.retain(|a| !a.belongs_to(root_group_id));
        flattened.sort();
        flattened
    }

    pub fn belongs_to(&self, group: &str) -> bool {
        if self.group_id.starts_with(group) {
            return if self.group_id.len() > group.len() {
                self.group_id.chars().nth(group.len()) == Some('.')
            } else {
                true
            } 
        }
        false
    }
}

fn add_artifacts_to_set(artifacts: Vec<Artifact>, set: &mut HashSet<Artifact>) {
    for a in artifacts {
        let isolated = Artifact {
            group_id: a.group_id,
            artifact_id: a.artifact_id,
            version: a.version,
            artifact_type: a.artifact_type,
            scope: a.scope,
            classifier: a.classifier,
            optional: a.optional,
            children: vec![],
            latest_version: a.latest_version,
        };
        add_artifacts_to_set(a.children, set);
        set.insert(isolated);
    }
}

pub fn read_tree<R: Read>(reader: R) -> Result<Artifact, Box<dyn Error>> {
    let buffered = BufReader::new(reader);
    let a = serde_json::from_reader(buffered)?;
    Ok(a)
}

pub fn write_as_csv<W: Write>(writer: W, artifacts: &Vec<Artifact>) -> Result<(), Box<dyn Error>> {
    let mut buffered = BufWriter::new(writer);
    for a in artifacts {
        writeln!(
            &mut buffered,
            "{},{},{},{}",
            a.group_id, a.artifact_id, a.version, a.latest_version
        )?;
    }
    Ok(())
}
