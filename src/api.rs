use std::collections::HashMap;
use std::error::Error;

use log::info;
use reqwest::{Url, blocking};
use serde::Deserialize;

use crate::artifact::Artifact;

pub struct RestClient {
    inner: blocking::Client,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Doc {
    id: String,
    g: String,
    a: String,
    #[serde(rename = "latestVersion")]
    latest_version: String,
}

#[derive(Deserialize, Debug)]
struct Response {
    docs: Vec<Doc>,
}

#[derive(Deserialize, Debug)]
struct Envelope {
    response: Response,
}

impl RestClient {
    const USER_AGENT: &'static str = "reqwest/0.13.2";
    const BASE_URL: &'static str = "https://search.maven.org/solrsearch/select";

    pub fn new() -> Result<RestClient, Box<dyn Error>> {
        let inner = blocking::Client::builder()
            .user_agent(Self::USER_AGENT)
            .build()?;
        Ok(RestClient { inner })
    }

    pub fn get_latest_version(&self, a: &mut Artifact) -> Result<(), Box<dyn Error>> {
        let mut params = HashMap::new();
        params.insert("q", format!("g:{} AND a:{}", a.group_id, a.artifact_id));
        params.insert("rows", "1".to_string());
        params.insert("wt", "json".to_string());
        let url = Url::parse_with_params(Self::BASE_URL, params)?;
        let resp = self.inner.get(url).send()?.error_for_status()?;
        let envelope: Envelope = serde_json::from_reader(resp)?;
        let docs = &envelope.response.docs;
        if docs.len() > 0 {
            a.latest_version = docs[0].latest_version.clone();
        }
        Ok(())
    }
}

pub fn fetch_latest_version(artifacts: &mut Vec<Artifact>) -> Result<(), Box<dyn Error>> {
    let client = RestClient::new()?;
    for a in artifacts {
        info!("Fetching metadata for {}:{}", a.group_id, a.artifact_id);
        client.get_latest_version(a)?;
    }
    Ok(())
}
