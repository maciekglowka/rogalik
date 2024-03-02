use std::error::Error;

#[derive(Debug)]
pub enum WorldError {
    EntityError,
    SerializationError(String),
    DeserializationError(String)
}

// #[derive(Debug)]
// pub struct EntityError;

// #[derive(Debug)]
// pub struct SerializationError;
// #[derive(Debug)]
// pub struct DeserializationError;