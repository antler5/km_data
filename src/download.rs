use serde::Deserialize;
use crate::{Data, Result, Error};
use directories::BaseDirs;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct GithubFileData {
    name: String,
    download_url: String,
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

impl From<minreq::Error> for Error {
    fn from(err: minreq::Error) -> Error {
        Error::Download(err)
    }
}

impl Data {
    pub fn with_download() -> Result<Self> {
        let base_dirs = BaseDirs::new().ok_or(Error::BaseDirs)?;
        let data_dir = base_dirs.data_dir().join("keymeow");
        if !data_dir.exists() {
            download_files(&data_dir)?;
        }
        Self::new()
    }
}

pub fn download_files(data_dir: &Path) -> Result<()> {
    download_repo(
        &data_dir.join("layouts"),
        "https://api.github.com/repos/semilin/km_layouts/contents/",
    )?;
    download_repo(
        &data_dir.join("metrics"),
        "https://api.github.com/repos/semilin/km_metric_data/contents/",
    )?;
    download_repo(
        &data_dir.join("corpora"),
        "https://api.github.com/repos/semilin/km_corpora/contents/",
    )?;
    Ok(())
}

fn get(url: &str) -> minreq::Request {
    minreq::get(url)
        .with_header("User-Agent", APP_USER_AGENT)
        .with_timeout(8)
}

fn download_repo(directory: &Path, url: &str) -> Result<()> {
    let resp = get(url).send();
    let data = resp?.json::<Vec<GithubFileData>>()?;

    for filedata in data {
        if let Ok(contents) = get(&filedata.download_url).send() {
            fs::write(directory.join(filedata.name), contents.as_bytes()).map_err(Error::FileWrite)?;
        };
    }
    Ok(())
}
