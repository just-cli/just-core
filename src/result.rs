use std::error;
use std::fmt;
use std::result;

pub type BoxedResult<T> = result::Result<T, Box<error::Error>>;

#[derive(Debug, Clone)]
pub struct BoxedErr {
    msg: String,
}

impl BoxedErr {
    pub fn with<T, S: Into<String>>(msg: S) -> BoxedResult<T> {
        let e = Self { msg: msg.into() };

        Err(e.into())
    }
}

impl fmt::Display for BoxedErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl error::Error for BoxedErr {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
