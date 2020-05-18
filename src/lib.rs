//! A crate to help you fetch and serve WebFinger resources.
//!
//! Use [`resolve`] to fetch remote resources, and [`Resolver`] to serve your own resources.

use reqwest::{header::ACCEPT, Client};
use serde::{Deserialize, Serialize};

mod sync_trait;
pub use crate::sync_trait::*;

#[cfg(feature = "async")]
mod async_trait;
#[cfg(feature = "async")]
pub use crate::async_trait::*;

#[cfg(test)]
mod tests;

/// WebFinger result that may serialized or deserialized to JSON
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub enum WebfingerError {
    /// The error came from the HTTP client.
    HttpError,

    /// The requested resource couldn't be parsed, and thus couldn't be fetched
    ParseError,

    /// The received JSON couldn't be parsed into a valid [`Webfinger`] struct.
    JsonError,
}

/// A prefix for a resource, either `acct:`, `group:` or some custom type.
#[derive(Debug, PartialEq)]
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

impl Into<String> for Prefix {
    fn into(self) -> String {
        match self {
            Prefix::Acct => "acct".into(),
            Prefix::Group => "group".into(),
            Prefix::Custom(x) => x.clone(),
        }
    }
}

/// Computes the URL to fetch for a given resource.
///
/// # Parameters
///
/// - `prefix`: the resource prefix
/// - `acct`: the identifier of the resource, for instance: `someone@example.org`
/// - `with_https`: indicates wether the URL should be on HTTPS or HTTP
///
pub fn url_for(
    prefix: Prefix,
    acct: impl Into<String>,
    with_https: bool,
) -> Result<String, WebfingerError> {
    let acct = acct.into();
    let scheme = if with_https { "https" } else { "http" };

    let prefix: String = prefix.into();
    acct.split("@")
        .nth(1)
        .ok_or(WebfingerError::ParseError)
        .map(|instance| {
            format!(
                "{}://{}/.well-known/webfinger?resource={}:{}",
                scheme, instance, prefix, acct
            )
        })
}

/// Fetches a WebFinger resource, identified by the `acct` parameter, a Webfinger URI.
pub async fn resolve_with_prefix(
    prefix: Prefix,
    acct: impl Into<String>,
    with_https: bool,
) -> Result<Webfinger, WebfingerError> {
    let url = url_for(prefix, acct, with_https)?;
    Client::new()
        .get(&url[..])
        .header(ACCEPT, "application/jrd+json, application/json")
        .send()
        .await
        .map_err(|_| WebfingerError::HttpError)?
        .json()
        .await
        .map_err(|_| WebfingerError::JsonError)
}

/// Fetches a Webfinger resource.
///
/// If the resource doesn't have a prefix, `acct:` will be used.
pub async fn resolve(
    acct: impl Into<String>,
    with_https: bool,
) -> Result<Webfinger, WebfingerError> {
    let acct = acct.into();
    let mut parsed = acct.splitn(2, ':');
    let first = parsed.next().ok_or(WebfingerError::ParseError)?;

    if first.contains('@') {
        // This : was a port number, not a prefix
        resolve_with_prefix(Prefix::Acct, acct, with_https).await
    } else {
        if let Some(other) = parsed.next() {
            resolve_with_prefix(Prefix::from(first), other, with_https).await
        } else {
            // fallback to acct:
            resolve_with_prefix(Prefix::Acct, first, with_https).await
        }
    }
}

/// An error that occured while handling an incoming WebFinger request.
#[derive(Debug, PartialEq)]
pub enum ResolverError {
    /// The requested resource was not correctly formatted
    InvalidResource,

    /// The website of the resource is not the current one.
    WrongDomain,

    /// The requested resource was not found.
    NotFound,
}
