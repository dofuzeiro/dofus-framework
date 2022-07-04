use std::fs;

use serde::Deserialize;
use thiserror::Error;

use crate::io::file::deserializer::DeserializationError::{InvalidInput, NotFound};

#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error("Format {0} yet to be implemented")]
    NotImplemented(String),
    #[error("Cannot deserialize data:\n\n{0}\n\nas {1} format")]
    InvalidInput(String, &'static str),
    #[error("No file found at {0}")]
    NotFound(String),
}

pub trait Deserializer {
    fn deserialize<E>(&self, data: String) -> Result<E, DeserializationError>
    where
        E: for<'a> Deserialize<'a>;
}

#[derive(Debug)]
pub enum Format {
    Json,
    Yaml,
    Xml,
    Toml,
}

impl Deserializer for Format {
    fn deserialize<E>(&self, data: String) -> Result<E, DeserializationError>
    where
        E: for<'a> Deserialize<'a>,
    {
        let data = data.as_str();
        match self {
            Format::Json => {
                serde_json::from_str(data).map_err(|_| InvalidInput(data.to_string(), "JSON"))
            }
            Format::Yaml => {
                serde_yaml::from_str(data).map_err(|_| InvalidInput(data.to_string(), "YAML"))
            }
            Format::Xml => {
                serde_xml_rs::from_str(data).map_err(|_| InvalidInput(data.to_string(), "XML"))
            }
            Format::Toml => {
                toml::from_str(data).map_err(|_| InvalidInput(data.to_string(), "TOML"))
            } //_ => Err(NotImplemented(format!("{:?}", self))),
        }
    }
}

pub fn deserialize_from_file<E, T>(
    file_path: &str,
    deserializer: T,
) -> Result<E, DeserializationError>
where
    E: for<'a> Deserialize<'a>,
    T: Deserializer,
{
    fs::read_to_string(file_path)
        .map_err(|_| NotFound(file_path.to_string()))
        .map(|file_data| deserializer.deserialize(file_data))?
}
