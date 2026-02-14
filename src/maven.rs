use std::{
    error::Error,
    io::{BufReader, Read},
};

use crate::artifact::{Artifact, TreeParser};

pub struct MavenTreeParser {}

impl TreeParser for MavenTreeParser {
    fn parse(&self, reader: &mut dyn Read) -> Result<Artifact, Box<dyn Error>> {
        let buffered = BufReader::new(reader);
        let a = serde_json::from_reader(buffered)?;
        Ok(a)
    }
}
