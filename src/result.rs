use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("Could not convert json value from type_url: {type_url}")]
  JsonTypeUrlUnknown { type_url: String },
  #[error("Could not unpack a non-optional value from null")]
  ValueNotPresent,
  #[error("Could not unpack field '{field_name}' from null")]
  FieldValueNotPresent { field_name: &'static str },
  #[error("JSON value nested too deeply")]
  JsonValueNestedTooDeeply,
  #[error("List element {index}: {source}")]
  ListElement { source: Box<Error>, index: usize },
  #[error("Map entry: {}", source)]
  MapEntry { source: Box<Error> },
  #[error("Parse decimal error: {0}")]
  ParseBigDecimal(#[from] bigdecimal::ParseBigDecimalError),
  #[error("Parse duration error: {message}")]
  ParseDuration { message: String },
  #[error(
    "Enum discriminant is not found: enum type = {}, discriminant = {}",
    enum_name,
    discriminant
  )]
  EnumDiscriminantNotFound {
    enum_name: &'static str,
    discriminant: i32,
  },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
