use std::result;

pub type BoxedResult<T> = result::Result<T, Box<std::error::Error>>;
