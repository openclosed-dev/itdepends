use std::{
    error::Error,
    io::{BufRead, BufReader, Lines, Read},
    iter::Peekable,
    vec,
};

use crate::artifact::{Artifact, ArtifactParser};

pub struct GradleArtifactParser<R: Read> {
    iter: Peekable<Lines<BufReader<R>>>,
}

impl<R: Read> GradleArtifactParser<R> {
    pub fn new(reader: R) -> GradleArtifactParser<R> {
        let buf_reader = BufReader::new(reader);
        GradleArtifactParser {
            iter: buf_reader.lines().peekable(),
        }
    }

    fn parse_configuration(&mut self) -> Result<Artifact, Box<dyn Error>> {
        let children = self.parse_children(0)?;
        Ok(create_root_artifact(children))
    }

    fn parse_children(&mut self, parent_level: usize) -> Result<Vec<Artifact>, Box<dyn Error>> {
        let mut children = vec![];
        let child_level = parent_level + 5;
        while let Some(result) = self.iter.next_if(|x| match x {
            Ok(line) => get_entry_level(line) > parent_level,
            Err(_) => true,
        }) {
            let line = result?;
            if let Some(mut child) = parse_line(&line, child_level) {
                child.children = self.parse_children(child_level)?;
                children.push(child);
            }
        }
        Ok(children)
    }
}

fn parse_line(line: &str, col: usize) -> Option<Artifact> {
    let mut entry = &line[col..];

    if let Some(index) = entry.find("(") {
        let remark = &entry[index..];
        // constraint
        if remark == "(c)" {
            return None;
        }
        entry = &entry[0..index].trim_end();
    }

    let parts: Vec<&str> = entry.splitn(2, "->").collect();
    let first = parts[0].trim_end();
    let coordinates: Vec<&str> = first.splitn(3, ":").collect();

    // group id may be 'project' followed by a whitespace
    let group_id = if coordinates[0] == "project " {
        ""
    } else {
        coordinates[0]
    };

    let artifact_id = coordinates[1];

    let version = if parts.len() >= 2 {
        parts[1].trim_start()
    } else if coordinates.len() >= 3 {
        coordinates[2]
    } else {
        ""
    };

    Some(Artifact {
        group_id: group_id.to_string(),
        artifact_id: artifact_id.to_string(),
        version: version.to_string(),
        scope: "runtime".to_string(),
        children: vec![],
        latest_version: None,
    })
}

fn get_entry_level(line: &str) -> usize {
    match line.find("--- ") {
        Some(index) => index + 4,
        None => 0,
    }
}

fn create_root_artifact(children: Vec<Artifact>) -> Artifact {
    Artifact {
        group_id: "".to_string(),
        artifact_id: "".to_string(),
        version: "".to_string(),
        scope: "runtime".to_string(),
        children: children,
        latest_version: None,
    }
}

impl<R: Read> ArtifactParser for GradleArtifactParser<R> {
    fn parse(&mut self) -> Result<Artifact, Box<dyn Error>> {
        while let Some(result) = self.iter.next() {
            let line = result?;
            if line.starts_with("runtimeClasspath")
                || line.starts_with("productionRuntimeClasspath")
            {
                return self.parse_configuration();
            }
        }
        Err("Runtime configuration is not found".to_string().into())
    }
}
