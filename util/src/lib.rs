use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{message:?}. Offending value: {offender:?}")]
    ADIFSerializeError { message: String, offender: String },
    #[error("Could not parse {field_name:?}. Offending value: {field_value:?}. More info: {err:?}")]
    FieldParseError {
        field_name: String,
        field_value: String,
        err: String,
    },
    #[error("Key {0:?} does not exist in database.")]
    DatabaseGetError(String)
}
