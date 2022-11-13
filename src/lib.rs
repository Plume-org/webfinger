//! A crate to help you fetch and serve WebFinger resources.
//!
//! Use [`resolve`] to fetch remote resources, and [`Resolver`] to serve your own resources.

use std::borrow::Cow;
use serde::{Deserialize, Serialize};

mod resolver;
pub use crate::resolver::*;

#[cfg(feature = "async")]
mod async_resolver;
#[cfg(feature = "async")]
pub use crate::async_resolver::*;

#[cfg(feature = "fetch")]
mod fetch;
#[cfg(feature = "fetch")]
pub use crate::fetch::*;

#[cfg(test)]
mod tests;

/// WebFinger result that may serialized or deserialized to JSON
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Webfinger {
    /// The subject of this WebFinger result.
    ///
    /// It is an `acct:` URI
    pub subject: String,

    /// A list of aliases for this WebFinger result.
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Links to places where you may find more information about this resource.
    pub links: Vec<Link>,
}

/// Structure to represent a WebFinger link
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Link {
    /// Tells what this link represents
    pub rel: String,

    /// The actual URL of the link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,

    /// The Link may also contain an URL template, instead of an actual URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// The mime-type of this link.
    ///
    /// If you fetch this URL, you may want to use this value for the Accept header of your HTTP
    /// request.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// An error that occured while fetching a WebFinger resource.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WebfingerError {
    /// The error came from the HTTP client.
    HttpError,

    /// The requested resource couldn't be parsed, and thus couldn't be fetched
    ParseError,

    /// The received JSON couldn't be parsed into a valid [`Webfinger`] struct.
    JsonError,
}

/// A prefix for a resource, either `acct:`, `group:` or some custom type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Prefix {
    /// `acct:` resource
    Acct,
    /// `group:` resource
    Group,
    /// Another type of resource
    Custom(String),
}

impl From<&str> for Prefix {
    fn from(s: &str) -> Prefix {
        match s.to_lowercase().as_ref() {
            "acct" => Prefix::Acct,
            "group" => Prefix::Group,
            x => Prefix::Custom(x.into()),
        }
    }
}

impl From<Prefix> for String {
    fn from(prefix: Prefix) -> Self {
        Cow::<'static, str>::from(prefix).into()
    }
}

impl From<Prefix> for Cow<'static, str> {
    fn from(prefix: Prefix) -> Self {
        match prefix {
            Prefix::Acct => "acct".into(),
            Prefix::Group => "group".into(),
            Prefix::Custom(x) => x.into(),
        }
    }
}

/// An error that occured while handling an incoming WebFinger request.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ResolverError {
    /// The requested resource was not correctly formatted
    InvalidResource,

    /// The website of the resource is not the current one.
    WrongDomain,

    /// The requested resource was not found.
    NotFound,
}
