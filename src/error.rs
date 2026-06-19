use std::fmt;

#[derive(Debug)]
pub enum HoldoutError {
    Io(std::io::Error),
    Json(serde_json::Error),
    SealMismatch,
}

impl fmt::Display for HoldoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HoldoutError::Io(e) => write!(f, "io error: {e}"),
            HoldoutError::Json(e) => write!(f, "oracle json error: {e}"),
            HoldoutError::SealMismatch => {
                write!(
                    f,
                    "oracle seal mismatch: the oracle file was modified after sealing"
                )
            }
        }
    }
}

impl std::error::Error for HoldoutError {}

impl From<std::io::Error> for HoldoutError {
    fn from(e: std::io::Error) -> Self {
        HoldoutError::Io(e)
    }
}

impl From<serde_json::Error> for HoldoutError {
    fn from(e: serde_json::Error) -> Self {
        HoldoutError::Json(e)
    }
}
