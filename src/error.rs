use std::fmt;

/// A type alias for `Result<T, YomuError>`.
pub type Result<T> = std::result::Result<T, YomuError>;

/// The error type for all yomu operations.
#[derive(Debug)]
pub enum YomuError {
    /// An HTTP request error from reqwest.
    Http(reqwest::Error),
    /// An I/O error.
    Io(std::io::Error),
    /// An image decoding/encoding error.
    Image(image::ImageError),
}

impl fmt::Display for YomuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YomuError::Http(e) => write!(f, "HTTP error: {e}"),
            YomuError::Io(e) => write!(f, "IO error: {e}"),
            YomuError::Image(e) => write!(f, "Image error: {e}"),
        }
    }
}

impl std::error::Error for YomuError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            YomuError::Http(e) => Some(e),
            YomuError::Io(e) => Some(e),
            YomuError::Image(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for YomuError {
    fn from(err: reqwest::Error) -> Self {
        YomuError::Http(err)
    }
}

impl From<std::io::Error> for YomuError {
    fn from(err: std::io::Error) -> Self {
        YomuError::Io(err)
    }
}

impl From<image::ImageError> for YomuError {
    fn from(err: image::ImageError) -> Self {
        YomuError::Image(err)
    }
}
