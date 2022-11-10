use reqwest::{header::ACCEPT, Client};

use crate::*;

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
  acct.split('@')
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
  } else if let Some(other) = parsed.next() {
      resolve_with_prefix(Prefix::from(first), other, with_https).await
  } else {
      // fallback to acct:
      resolve_with_prefix(Prefix::Acct, first, with_https).await
  }
}