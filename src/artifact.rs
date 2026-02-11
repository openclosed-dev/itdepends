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
    group_id: String,
    #[serde(rename = "artifactId")]
    artifact_id: String,
    version: String,
    #[serde(rename = "type")]
    artifact_type: String,
    scope: String,
    classifier: String,
    #[serde(deserialize_with = "bool_from_string")]
    optional: bool,
    #[serde(default)]
    children: Vec<Artifact>,
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
        "true" => Ok(false),
        "false" => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Str(other),
            &"true of false",
        )),
    }
}

impl Artifact {
    pub fn flatten(&self) -> Vec<Artifact> {
        let mut set: HashSet<Artifact> = HashSet::new();
        self.add_children_to_set(&mut set);
        let mut flattened: Vec<Artifact> = set.into_iter().collect();
        flattened.sort();
        flattened
    }

    fn add_children_to_set(&self, set: &mut HashSet<Artifact>) {
        for child in &self.children {
            child.add_children_to_set(set);
            set.insert(child.without_children());
        }
    }

    fn without_children(&self) -> Artifact {
        Artifact {
            group_id: self.group_id.clone(),
            artifact_id: self.artifact_id.clone(),
            version: self.version.clone(),
            artifact_type: self.artifact_type.clone(),
            scope: self.scope.clone(),
            classifier: self.classifier.clone(),
            optional: self.optional,
            children: vec![],
        }
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
            "{},{},{}",
            a.group_id, a.artifact_id, a.version
        )?;
    }
    Ok(())
}
