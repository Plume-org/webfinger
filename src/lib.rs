extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use reqwest::{Client, header::{Accept, qitem}, mime::Mime};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Webfinger {
    subject: String,
    aliases: Vec<String>,
    links: Vec<Link>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Link {
    rel: String,
    href: String,
    #[serde(rename="type")]
    mime_type: Option<String>
}

#[derive(Debug, PartialEq)]
pub enum WebfingerError {
    HttpError,
    ParseError,
    JsonError
}

pub fn url_for_acct<T: Into<String>>(acct: T) -> Result<String, WebfingerError> {
    let acct = acct.into();
    acct.split("@")
        .nth(1)
        .ok_or(WebfingerError::ParseError)
        .map(|instance| format!("https://{}/.well-known/webfinger?resource=acct:{}", instance, acct))
}

pub fn resolve(acct: String) -> Result<Webfinger, WebfingerError> {
    let url = url_for_acct(acct)?;
    Client::new()
        .get(&url[..])
        .header(Accept(vec![qitem("application/jrd+json".parse::<Mime>().unwrap())]))
        .send()
        .map_err(|_| WebfingerError::HttpError)
        .and_then(|mut r| r.text().map_err(|_| WebfingerError::HttpError))
        .and_then(|res| serde_json::from_str(&res[..]).map_err(|_| WebfingerError::JsonError))
}

#[derive(Debug, PartialEq)]
pub enum ResolverError {
    InvalidResource,
    WrongInstance,
    NotFound
}

pub trait Resolver<R> {
    fn instance_domain<'a>() -> &'a str;
    fn find(acct: String, resource_repo: R) -> Result<Webfinger, ResolverError>;

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
