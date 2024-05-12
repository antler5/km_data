#[cfg(feature = "download")]
mod download;
use directories::BaseDirs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{error, fmt, fs, io};

#[derive(Debug)]
pub enum DataKind {
    Corpus,
    Keyboard,
    Layout,
}

impl fmt::Display for DataKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

#[derive(Debug)]
pub enum Error {
    BaseDirs,
    DirCreate(io::Error),
    DirRead(io::Error),
    FileRead(io::Error),
    FileWrite(io::Error),
    Locate(DataKind, String),
    RmpDeserialize(rmp_serde::decode::Error),
    JsonDeserialize(serde_json::Error),
    #[cfg(feature = "download")]
    Download(minreq::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BaseDirs => {
                write!(f, "directories crate could not find user's home directory")
            }
            Error::DirCreate(..) => write!(f, "could not create data directory"),
            Error::DirRead(..) => write!(f, "could not read data directory"),
            Error::FileRead(..) => write!(f, "could not read data file"),
            Error::FileWrite(..) => write!(f, "could not write file"),
            Error::Locate(kind, name) => write!(f, "could not find {kind} called `{name}`"),
            Error::RmpDeserialize(..) => write!(f, "error deserializing messagepack data"),
            Error::JsonDeserialize(..) => write!(f, "error deserializing json data"),
            #[cfg(feature = "download")]
            Error::Download(..) => write!(f, "error downloading data"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::DirRead(ref e) => Some(e),
            Error::DirCreate(ref e) => Some(e),
            Error::FileRead(ref e) => Some(e),
            Error::FileWrite(ref e) => Some(e),
            Error::RmpDeserialize(ref e) => Some(e),
            Error::JsonDeserialize(ref e) => Some(e),
            #[cfg(feature = "download")]
            Error::Download(ref e) => Some(e),
            _ => None,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct Data {
    pub data_dir: PathBuf,
    #[cfg(feature = "corpora")]
    pub corpora: HashMap<String, PathBuf>,
    #[cfg(feature = "keyboards")]
    pub keyboards: HashMap<String, PathBuf>,
    #[cfg(feature = "layouts")]
    pub layouts: HashMap<String, PathBuf>,
}

fn dir_to_hashmap(dir: &Path) -> Result<HashMap<String, PathBuf>> {
    Ok(fs::read_dir(dir)
        .map_err(Error::DirRead)?
        .filter_map(|x| {
            Some((
                x.as_ref()
                    .ok()?
                    .path()
                    .file_stem()?
                    .to_string_lossy()
                    .into_owned(),
                x.ok()?.path(),
            ))
        })
        .collect())
}

impl Data {
    fn create_directories(data_dir: &Path) -> Result<()> {
        for p in [
            &data_dir.join("corpora"),
            &data_dir.join("metrics"),
            &data_dir.join("layouts"),
        ] {
            fs::create_dir_all(p).map_err(Error::DirCreate)?;
        }
        Ok(())
    }
    pub fn new() -> Result<Self> {
        let base_dirs = BaseDirs::new().ok_or(Error::BaseDirs)?;
        let data_dir = base_dirs.data_dir().join("keymeow");

        Self::create_directories(&data_dir)?;

        Ok(Self {
            #[cfg(feature = "corpora")]
            corpora: dir_to_hashmap(&data_dir.join("corpora"))?,
            #[cfg(feature = "keyboards")]
            keyboards: dir_to_hashmap(&data_dir.join("metrics"))?,
            #[cfg(feature = "layouts")]
            layouts: dir_to_hashmap(&data_dir.join("layouts"))?,
            data_dir,
        })
    }
    #[cfg(feature = "corpora")]
    pub fn get_corpus(&self, s: &str) -> Result<keycat::Corpus> {
        let path = self
            .corpora
            .get(s)
            .ok_or_else(|| Error::Locate(DataKind::Corpus, s.to_owned()))?;
        let b = fs::read(path).map_err(Error::FileRead)?;
        rmp_serde::from_slice(&b).map_err(Error::RmpDeserialize)
    }
    #[cfg(feature = "keyboards")]
    pub fn get_metrics(&self, s: &str) -> Result<keymeow::MetricData> {
        let path = self
            .keyboards
            .get(s)
            .ok_or_else(|| Error::Locate(DataKind::Keyboard, s.to_owned()))?;
        let b = fs::read(path).map_err(Error::FileRead)?;
        rmp_serde::from_slice(&b).map_err(Error::RmpDeserialize)
    }
    #[cfg(feature = "layouts")]
    pub fn get_layout(&self, s: &str) -> Result<keymeow::LayoutData> {
        let path = self
            .keyboards
            .get(s)
            .ok_or_else(|| Error::Locate(DataKind::Keyboard, s.to_owned()))?;
        let b = fs::read_to_string(path).map_err(Error::FileRead)?;
        serde_json::from_str(&b).map_err(Error::JsonDeserialize)
    }
}
