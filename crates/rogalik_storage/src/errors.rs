#[derive(Debug)]
pub enum WorldError {
    EntityError,
    SerializationError(String),
    DeserializationError(String),
}
