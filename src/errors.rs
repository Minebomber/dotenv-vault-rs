use std::{error, fmt};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    KeyNotFound,
    VaultNotFound,
    DotenvyError(dotenvy::Error),
    ParseError(url::ParseError),
    InvalidScheme,
    MissingKey,
    MissingEnvironment,
    EnvironmentNotFound(String),
    InvalidKey,
    HexError(hex::FromHexError),
    DecodeError(base64::DecodeError),
    DecryptError(aes_gcm::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::KeyNotFound => {
                write!(
                    f,
                    "NOT_FOUND_DOTENV_KEY: Cannot find environment variable 'DOTENV_KEY'"
                )
            }
            Error::VaultNotFound => {
                write!(f, "NOT_FOUND_DOTENV_VAULT: Cannot find vault file")
            }
            Error::DotenvyError(ref error) => error.fmt(f),
            Error::ParseError(_) => {
                write!(f, "INVALID_DOTENV_KEY: Failed to parse url")
            }
            Error::InvalidScheme => {
                write!(f, "INVALID_DOTENV_KEY: Invalid scheme")
            }
            Error::MissingKey => {
                write!(f, "INVALID_DOTENV_KEY: Missing key part")
            }
            Error::MissingEnvironment => {
                write!(f, "INVALID_DOTENV_KEY: Missing environment part")
            }
            Error::EnvironmentNotFound(ref environment) => {
                write!(f, "NOT_FOUND_DOTENV_ENVIRONMENT: Cannot locate environment {} in your .env.vault file. Run 'npx dotenv-vault build' to include it.", environment)
            }
            Error::InvalidKey => {
                write!(f, "INVALID_DOTENV_KEY: Key must be valid")
            }
            Error::HexError(_) => {
                write!(f, "INVALID_DOTENV_KEY: Failed to decode hex string")
            }
            Error::DecodeError(_) => {
                write!(f, "DECRYPTION_FAILED: Failed to decode base64 string")
            }
            Error::DecryptError(_) => {
                write!(f, "DECRYPTION_FAILED: Please check your DOTENV_KEY")
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::KeyNotFound => None,
            Error::VaultNotFound => None,
            Error::DotenvyError(ref e) => Some(e),
            Error::ParseError(ref e) => Some(e),
            Error::InvalidScheme => None,
            Error::MissingKey => None,
            Error::MissingEnvironment => None,
            Error::EnvironmentNotFound(_) => None,
            Error::InvalidKey => None,
            Error::HexError(ref e) => Some(e),
            Error::DecodeError(ref e) => Some(e),
            Error::DecryptError(_) => None,
        }
    }
}

impl From<dotenvy::Error> for Error {
    fn from(err: dotenvy::Error) -> Error {
        Error::DotenvyError(err)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Error {
        Error::HexError(err)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Error {
        Error::DecodeError(err)
    }
}

impl From<aes_gcm::Error> for Error {
    fn from(err: aes_gcm::Error) -> Error {
        Error::DecryptError(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::ParseError(err)
    }
}
