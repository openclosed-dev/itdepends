use std::thread;
use std::{error::Error, time};

use log::info;
use reqwest::{Url, blocking};
use serde::Deserialize;

use crate::artifact::Artifact;

pub struct RestClient {
    inner: blocking::Client,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Versions {
    #[serde(rename = "version")]
    versions: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Versioning {
    latest: String,
    release: String,
    versions: Versions,
    #[serde(rename = "lastUpdated")]
    last_updated: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Metadata {
    #[serde(rename = "groupId")]
    group_id: String,
    #[serde(rename = "artifactId")]
    artifact_id: String,
    versioning: Versioning,
}

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

impl RestClient {
    const BASE_URL: &'static str = "https://repo.maven.apache.org/maven2";

    pub fn new() -> Result<RestClient, Box<dyn Error>> {
        let timeout = time::Duration::from_secs(180);
        let inner = blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(timeout)
            .build()?;
        Ok(RestClient { inner })
    }

    pub fn get_latest_version(&self, a: &mut Artifact) -> Result<(), Box<dyn Error>> {
        let mut url = Url::parse(Self::BASE_URL)?;
        {
            let mut segments = url.path_segments_mut().map_err(|_| "invalid URL")?;
            segments
                .push(&a.group_id.replace(".", "/"))
                .push(&&a.artifact_id)
                .push("maven-metadata.xml");
        }
        let resp = self.inner.get(url).send()?.error_for_status()?;
        let metadata: Metadata = serde_xml_rs::from_reader(resp)?;
        a.latest_version = Some(metadata.versioning.latest);
        Ok(())
    }
}

pub fn fetch_latest_version(artifacts: &mut Vec<Artifact>) -> Result<(), Box<dyn Error>> {
    let client = RestClient::new()?;

    let time_to_sleep = time::Duration::from_millis(1000);

    for (index, a) in artifacts.iter_mut().enumerate() {
        if index > 0 {
            thread::sleep(time_to_sleep);
        }
        info!("Fetching metadata for {}:{}", a.group_id, a.artifact_id);
        client.get_latest_version(a)?;
        let latest_version = a.latest_version.as_ref().map_or("none", |v| &v);
        info!("Fetched latest version: {}", latest_version);
    }
    Ok(())
}
