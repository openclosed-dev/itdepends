use std::{
    cmp::Ordering,
    collections::HashSet,
    error::Error,
    hash::{Hash, Hasher},
    io::{BufWriter, Read, Write},
};

#[derive(Clone, Debug)]
pub struct Artifact {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: String,
    pub children: Vec<Artifact>,
    pub latest_version: Option<String>,
}

impl PartialEq for Artifact {
    fn eq(&self, other: &Self) -> bool {
        self.group_id == other.group_id
            && self.artifact_id == other.artifact_id
            && self.version == other.version
    }
}

impl Eq for Artifact {}

impl Ord for Artifact {
    fn cmp(&self, other: &Self) -> Ordering {
        self.group_id
            .cmp(&other.group_id)
            .then(self.artifact_id.cmp(&other.artifact_id))
            .then(self.version.cmp(&other.version))
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
    }
}

impl Artifact {
    pub fn flatten(self) -> Vec<Artifact> {
        let mut set: HashSet<Artifact> = HashSet::new();
        add_artifacts_to_set(self.children, &mut set);
        let mut flattened: Vec<Artifact> = set.into_iter().collect();
        flattened.sort();
        flattened
    }

    pub fn belongs_to(&self, group: &str) -> bool {
        if self.group_id.starts_with(group) {
            return if self.group_id.len() > group.len() {
                self.group_id.chars().nth(group.len()) == Some('.')
            } else {
                true
            };
        }
        false
    }

    pub fn is_runtime(&self) -> bool {
        self.scope == "runtime" || self.scope == "compile"
    }
}

pub trait TreeParser {
    fn parse(&self, reader: &mut dyn Read) -> Result<Artifact, Box<dyn Error>>;
}

fn add_artifacts_to_set(artifacts: Vec<Artifact>, set: &mut HashSet<Artifact>) {
    for a in artifacts {
        let isolated = Artifact {
            group_id: a.group_id,
            artifact_id: a.artifact_id,
            version: a.version,
            scope: a.scope,
            children: vec![],
            latest_version: a.latest_version,
        };
        add_artifacts_to_set(a.children, set);
        set.insert(isolated);
    }
}

pub fn write_as_csv<W: Write>(writer: W, artifacts: &Vec<Artifact>) -> Result<(), Box<dyn Error>> {
    let mut buffered = BufWriter::new(writer);
    for a in artifacts {
        let latest_version = a.latest_version.as_deref().unwrap_or_default();
        writeln!(
            &mut buffered,
            "{},{},{},{}",
            a.group_id, a.artifact_id, a.version, latest_version
        )?;
    }
    Ok(())
}
