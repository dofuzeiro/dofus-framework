use serde::Deserialize;
use std::fs;
use thiserror::Error;

use crate::io::file::deserializer::DeserializationError::{EmptyFile, IOError, InvalidInput};

#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error("Format {0} yet to be implemented")]
    NotImplemented(String),
    #[error("Cannot deserialize data:\n\n{0}\n\nas {1} format")]
    InvalidInput(String, &'static str),
    #[error("File {0} is empty")]
    EmptyFile(String),
    #[error("Error while trying to read the contents of the file")]
    IOError {
        path: String,
        source: std::io::Error,
    },
}

pub trait Deserializer {
    fn deserialize<E>(&self, data: &str) -> Result<E, DeserializationError>
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
    fn deserialize<E>(&self, data: &str) -> Result<E, DeserializationError>
    where
        E: for<'a> Deserialize<'a>,
    {
        match self {
            Format::Json => deserialize_with_context(|x| serde_json::from_str(x), data, "JSON"),
            Format::Yaml => deserialize_with_context(serde_yaml::from_str, data, "YAML"),
            Format::Xml => deserialize_with_context(serde_xml_rs::from_str, data, "XML"),
            Format::Toml => deserialize_with_context(|x| toml::from_str(x), data, "TOML"),
            //_ => Err(NotImplemented(format!("{:?}", self))),
        }
    }
}

fn deserialize_with_context<T, E>(
    //    result: Result<T, E>,
    result: fn(&str) -> Result<T, E>,
    data: &str,
    format: &'static str,
) -> Result<T, DeserializationError> {
    result(data).map_err(|_| InvalidInput(data.to_string(), format))
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
        .map_err(|err| IOError {
            path: file_path.to_string(),
            source: err,
        })
        .map(|file_data| {
            if file_data.is_empty() {
                return Err(EmptyFile(file_path.to_string()));
            }
            deserializer.deserialize(file_data.as_str())
        })?
}
