use http::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use std::any::Any;
use std::hash::Hash;
use std::str::FromStr;
use std::{convert::Infallible, path::PathBuf};
use std::{io, sync::Arc};
use thiserror::Error;

use super::InputContent;
use crate::Uri;

/// Errors encountered when using the library Some external error types don't
/// implement `Serialize`. In such a case we represent the errors as strings to
/// avoid depending on the implementation details of the external errors.
/// Using `Arc` for some variants so they can be `Send`
#[serde_as]
#[derive(Error, Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub enum ErrorKind {
    // TODO: maybe needs to be split; currently first element is `Some` only for
    // reading files
    /// Any form of I/O error occurred while reading from a given path.
    #[error("Failed to read from path: `{}`, reason: {1}", match .0 {
        Some(p) => p.to_str().unwrap_or("<MALFORMED PATH>"),
        None => "<MALFORMED PATH>",
    })]
    #[serde(
        serialize_with = "io_error_serialize",
        deserialize_with = "io_error_deserialize"
    )]
    // #[serde_as(as = "DisplayFromStr")]
    // #[serde(
    //     serialize_with = "serde_with::rust::display_fromstr::serialize",
    //     deserialize_with = "never_invoke"
    // )]
    // #[serde(deserialize_with = "string_deserialize")]
    // Io(Option<PathBuf>, #[serde(with = "serde_with::rust::display_fromstr")] Arc<io::Error>),
    // Io(Option<PathBuf>, #[serde_as(as = "DisplayFromStr")] Arc<io::Error>),
    Io(Option<PathBuf>, Arc<io::Error>),
    /// Errors which can occur when attempting to interpret a sequence of u8 as a string
    #[error("Attempted to interpret an invalid sequence of bytes as a string: {0}")]
    #[serde(skip)]
    Encoding(#[from] std::str::Utf8Error),
    /// Reqwest network error
    #[error("Network error while trying to connect to an endpoint: {err}")]
    // #[serde(
    //     serialize_with = "client_error_serialize",
    //     deserialize_with = "client_error_deserialize"
    // )]
    Client {
        /// Reqwest error message
        err: String,
        /// Optional status code
        #[serde(
            serialize_with = "serialize_statuscode",
            deserialize_with = "deserialize_statuscode"
        )]
        status: Option<StatusCode>,
    },
    /// Hubcaps network error
    #[error("Network error when trying to connect to an endpoint via hubcaps: {0}")]
    Github(String),
    /// The given string can not be parsed into a valid URL, e-mail address, or file path
    #[error("Cannot parse {0} as website url, file path, or mail address: ({1:?})")]
    #[serde(skip)]
    Parse(
        String,
        (url::ParseError, Option<Arc<fast_chemail::ParseError>>),
    ),
    /// The given path cannot be converted to a URI
    #[error("Invalid path to URL conversion: {0}")]
    UrlFromPath(PathBuf),
    /// The given path does not resolve to a valid file
    #[error("Cannot find local file {0}")]
    FileNotFound(PathBuf),
    /// The given URI cannot be does not point to a valid file
    #[error("Cannot find file {0}")]
    FileUriNotFound(Uri),
    /// Mail address is unreachable
    #[error("Unreachable mail address: {0}")]
    Mail(Uri),
    /// The given header could not be parsed.
    /// A possible error when converting a `HeaderValue` from a string or byte
    /// slice.
    #[error("Header could not be parsed: {0}")]
    #[serde(skip)]
    Header(#[from] Arc<http::header::InvalidHeaderValue>),
    /// The given string can not be parsed into a valid base URL or base directory
    #[error("Error with base dir `{0}` : {1}")]
    Base(String, String),
    /// Error while traversing an input directory
    #[error("Cannot traverse input directory: {0}")]
    #[serde(skip)]
    DirTraversal(#[from] Arc<jwalk::Error>),
    /// The given glob pattern is not valid
    #[error("UNIX glob pattern is invalid")]
    #[serde(skip)]
    Glob(#[from] Arc<glob::PatternError>),
    /// The Github API could not be called because of a missing Github token.
    #[error("GitHub token not specified. To check GitHub links reliably, use `--github-token` flag / `GITHUB_TOKEN` env var.")]
    MissingGitHubToken,
    /// Used an insecure URI where a secure variant was reachable
    #[error("This URI is available in HTTPS protocol, but HTTP is provided, use '{0}' instead")]
    InsecureURL(Uri),
    /// Error while sending/receiving messages from MPSC channel
    #[error("Cannot send/receive message from channel: {0}")]
    #[serde(skip)]
    Channel(#[from] Arc<tokio::sync::mpsc::error::SendError<InputContent>>),
    /// An URL with an invalid host was found
    #[error("URL is missing a host")]
    MissingHost,
    /// Cannot parse the given URI
    #[error("The given URI is invalid: {0}")]
    InvalidURI(Uri),
}

fn io_error_serialize<S>(_path: &Option<PathBuf>, e: &io::Error, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&e.to_string())
}

fn io_error_deserialize<'de, D>(
    deserializer: D,
) -> Result<(Option<PathBuf>, Arc<io::Error>), D::Error>
where
    D: Deserializer<'de>,
{
    let msg = String::deserialize(deserializer)?;
    Ok((
        None,
        Arc::new(io::Error::new(std::io::ErrorKind::Other, msg)),
    ))
}

fn serialize_statuscode<S>(status: &Option<StatusCode>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match status.map(|c| c.as_u16()) {
        Some(ref value) => s.serialize_some(value),
        None => s.serialize_none(),
    }
}

fn deserialize_statuscode<'de, D>(deserializer: D) -> Result<Option<StatusCode>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        return Ok(Some(
            StatusCode::from_str(&s).map_err(serde::de::Error::custom)?,
        ));
    };
    Ok(None)
}

// fn client_error_serialize<S>(e: &str, status: &Option<u32>, s: S) -> Result<S::Ok, S::Error>
// where
//     S: serde::Serializer,
// {
//     s.serialize_str(&e.to_string())
// }

// fn client_error_deserialize<'de, D>(
//     deserializer: D,
// ) -> Result<Arc<reqwest::Error>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let msg = String::deserialize(deserializer)?;
//     Ok((
//         None,
//         Arc::new(reqwest::Error::new(std::io::ErrorKind::Other, msg)),
//     ))
// }

impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(p1, e1), Self::Io(p2, e2)) => p1 == p2 && e1.kind() == e2.kind(),
            (
                Self::Client {
                    err: l_err,
                    status: l_status,
                },
                Self::Client {
                    err: r_err,
                    status: r_status,
                },
            ) => l_err == r_err && l_status == r_status,
            (Self::Github(e1), Self::Github(e2)) => e1.to_string() == e2.to_string(),
            (Self::Parse(s1, e1), Self::Parse(s2, e2)) => s1 == s2 && e1 == e2,
            (Self::Mail(u1), Self::Mail(u2)) | (Self::InsecureURL(u1), Self::InsecureURL(u2)) => {
                u1 == u2
            }
            (Self::Glob(e1), Self::Glob(e2)) => e1.msg == e2.msg && e1.pos == e2.pos,
            (Self::Header(_), Self::Header(_))
            | (Self::MissingGitHubToken, Self::MissingGitHubToken) => true,
            _ => false,
        }
    }
}

impl Eq for ErrorKind {}

impl Hash for ErrorKind {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        match self {
            Self::Io(p, e) => (p, e.kind()).hash(state),
            Self::Client { err, status } => (err, status).hash(state),
            Self::Github(e) => e.to_string().hash(state),
            Self::DirTraversal(e) => e.to_string().hash(state),
            Self::FileNotFound(e) => e.to_string_lossy().hash(state),
            Self::Parse(s, e) => (s, e.type_id()).hash(state),
            Self::InvalidURI(u) => u.hash(state),
            Self::UrlFromPath(p) => p.hash(state),
            Self::Encoding(e) => e.to_string().hash(state),
            Self::FileUriNotFound(u) | Self::Mail(u) | Self::InsecureURL(u) => {
                u.hash(state);
            }
            Self::Base(base, e) => (base, e).hash(state),
            Self::Header(e) => e.to_string().hash(state),
            Self::Glob(e) => e.to_string().hash(state),
            Self::Channel(e) => e.to_string().hash(state),
            Self::MissingGitHubToken | Self::MissingHost => {
                std::mem::discriminant(self).hash(state);
            }
        }
    }
}

// impl Serialize for ErrorKind {
//     fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.collect_str(self)
//     }
// }

impl From<(PathBuf, std::io::Error)> for ErrorKind {
    fn from(value: (PathBuf, std::io::Error)) -> Self {
        Self::Io(Some(value.0), Arc::new(value.1))
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        Self::Io(None, Arc::new(e))
    }
}

impl From<tokio::task::JoinError> for ErrorKind {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::Io(None, Arc::new(e.into()))
    }
}

impl From<url::ParseError> for ErrorKind {
    fn from(e: url::ParseError) -> Self {
        Self::Parse("Cannot parse URL".to_string(), (e, None))
    }
}

impl From<(String, url::ParseError)> for ErrorKind {
    fn from(value: (String, url::ParseError)) -> Self {
        Self::Parse(value.0, (value.1, None))
    }
}

impl From<(String, url::ParseError, fast_chemail::ParseError)> for ErrorKind {
    fn from(value: (String, url::ParseError, fast_chemail::ParseError)) -> Self {
        Self::Parse(value.0, (value.1, Some(Arc::new(value.2))))
    }
}

impl From<Infallible> for ErrorKind {
    fn from(_: Infallible) -> Self {
        // tautological
        unreachable!()
    }
}
