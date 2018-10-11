//! A crate to help you fetch and serve WebFinger resources.
//! 
//! Use [`resolve`] to fetch remote resources, and [`Resolver`] to serve your own resources.

extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use reqwest::{header::ACCEPT, Client};

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
    pub aliases: Vec<String>,

    /// Links to places where you may find more information about this resource.
    pub links: Vec<Link>
}

/// Structure to represent a WebFinger link
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Link {
    /// Tells what this link represents
    pub rel: String,

    /// The actual URL of the link
    pub href: String,

    /// The mime-type of this link.
    /// 
    /// If you fetch this URL, you may want to use this value for the Accept header of your HTTP
    /// request.
    #[serde(rename="type")]
    pub mime_type: Option<String>
}

/// An error that occured while fetching a WebFinger resource.
#[derive(Debug, PartialEq)]
pub enum WebfingerError {
    /// The error came from the HTTP client.
    HttpError,

    /// The requested resource couldn't be parsed, and thus couldn't be fetched
    ParseError,

    /// The received JSON couldn't be parsed into a valid [`Webfinger`] struct.
    JsonError
}

/// Computes the URL to fetch for an `acct:` URI.
/// 
/// # Example
/// 
/// ```rust
/// use webfinger::url_for_acct;
/// 
/// assert_eq!(url_for_acct("test@example.org"), Ok(String::from("https://example.org/.well-known/webfinger?resource=acct:test@example.org")));
/// ```
pub fn url_for_acct<T: Into<String>>(acct: T, with_https: bool) -> Result<String, WebfingerError> {
    let acct = acct.into();
    let scheme = if with_https {
        "https"
    } else {
        "http"
    };

    acct.split("@")
        .nth(1)
        .ok_or(WebfingerError::ParseError)
        .map(|instance| format!("{}://{}/.well-known/webfinger?resource=acct:{}", scheme, instance, acct))
}

/// Fetches a WebFinger resource, identified by the `acct` parameter, an `acct:` URI.
pub fn resolve<T: Into<String>>(acct: T, with_https: bool) -> Result<Webfinger, WebfingerError> {
    let url = url_for_acct(acct, with_https)?;
    Client::new()
        .get(&url[..])
        .header(ACCEPT, "application/jrd+json")
        .send()
        .map_err(|_| WebfingerError::HttpError)
        .and_then(|mut r| r.text().map_err(|_| WebfingerError::HttpError))
        .and_then(|res| serde_json::from_str(&res[..]).map_err(|_| WebfingerError::JsonError))
}

/// An error that occured while handling an incoming WebFinger request.
#[derive(Debug, PartialEq)]
pub enum ResolverError {
    /// The requested resource was not correctly formatted
    InvalidResource,

    /// The website of the resource is not the current one.
    WrongInstance,

    /// The requested resource was not found.
    NotFound
}

/// A trait to easily generate a WebFinger endpoint for any resource repository.
/// 
/// The `R` type is your resource repository (a database for instance) that will be passed to the
/// [`find`](Resolver::find) and [`endpoint`](Resolver::endpoint) functions.
pub trait Resolver<R> {
    /// Returns the domain name of the current instance.
    fn instance_domain<'a>() -> &'a str;

    /// Tries to find a resource, `acct`, in the repository `resource_repo`.
    /// 
    /// `acct` is not a complete `acct:` URI, it only contains the identifier of the requested resource
    /// (e.g. `test` for `acct:test@example.org`)
    /// 
    /// If the resource couldn't be found, you may probably want to return a [`ResolverError::NotFound`].
    fn find(acct: String, resource_repo: R) -> Result<Webfinger, ResolverError>;

    /// Returns a WebFinger result for a requested resource.
    fn endpoint<T: Into<String>>(resource: T, resource_repo: R) -> Result<Webfinger, ResolverError> {
        let resource = resource.into();
        let mut parsed_query = resource.splitn(2, ":");
        parsed_query.next()
            .ok_or(ResolverError::InvalidResource)
            .and_then(|res_type| {
                if res_type == "acct" {
                    parsed_query.next().ok_or(ResolverError::InvalidResource)
                } else {
                    Err(ResolverError::InvalidResource)
                }
            })
            .and_then(|res| {
                let mut parsed_res = res.split("@");
                parsed_res.next()
                    .ok_or(ResolverError::InvalidResource)
                    .and_then(|user| {
                        parsed_res.next()
                            .ok_or(ResolverError::InvalidResource)
                            .and_then(|res_domain| {
                                if res_domain == Self::instance_domain() {
                                    Self::find(user.to_string(), resource_repo)
                                } else {
                                    Err(ResolverError::WrongInstance)
                                }
                            })
                    })
            })
    }
}
