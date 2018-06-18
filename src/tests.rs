use serde_json;
use super::*;

#[test]
fn test_url_for_acct() {
    assert_eq!(url_for_acct("test@example.org"), Ok(String::from("https://example.org/.well-known/webfinger?resource=acct:test@example.org")));
    assert_eq!(url_for_acct("test"), Err(WebfingerError::ParseError))
}

#[test]
fn test_webfinger_parsing() {
    let valid = r#"
    {
        "subject": "acct:test@example.org",
        "aliases": [
            "https://example.org/@test/"
        ],
        "links": [
            {
                "rel": "http://webfinger.net/rel/profile-page",
                "href": "https://example.org/@test/"
            },
            {
                "rel": "http://schemas.google.com/g/2010#updates-from",
                "type": "application/atom+xml",
                "href": "https://example.org/@test/feed.atom"
            },
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": "https://example.org/@test/"
            }
        ]
    }
    "#;
    let webfinger: Webfinger = serde_json::from_str(valid).unwrap();
    assert_eq!(String::from("acct:test@example.org"), webfinger.subject);
    assert_eq!(vec!["https://example.org/@test/"], webfinger.aliases);
    assert_eq!(vec![
        Link {
            rel: "http://webfinger.net/rel/profile-page".to_string(),
            mime_type: None,
            href: "https://example.org/@test/".to_string()
        },
        Link {
            rel: "http://schemas.google.com/g/2010#updates-from".to_string(),
            mime_type: Some("application/atom+xml".to_string()),
            href: "https://example.org/@test/feed.atom".to_string()
        },
        Link {
            rel: "self".to_string(),
            mime_type: Some("application/activity+json".to_string()),
            href: "https://example.org/@test/".to_string()
        }
    ], webfinger.links);
}

pub struct MyResolver;

// Only one user, represented by a String
impl Resolver<&'static str> for MyResolver {
    fn instance_domain<'a>() -> &'a str {
        "instance.tld"
    }

    fn find(acct: String, resource_repo: &'static str) -> Result<Webfinger, ResolverError> {
        if acct == resource_repo.to_string() {
            Ok(Webfinger {
                subject: acct.clone(),
                aliases: vec![acct.clone()],
                links: vec![
                    Link {
                        rel: "http://webfinger.net/rel/profile-page".to_string(),
                        mime_type: None,
                        href: format!("https://instance.tld/@{}/", acct)
                    }
                ]
            })
        } else {
            Err(ResolverError::NotFound)
        }
    }
}

#[test]
fn test_my_resolver() {
    assert!(MyResolver::endpoint("acct:admin@instance.tld", "admin").is_ok());
    assert_eq!(MyResolver::endpoint("acct:test@instance.tld", "admin"), Err(ResolverError::NotFound));
    assert_eq!(MyResolver::endpoint("acct:admin@oops.ie", "admin"), Err(ResolverError::WrongInstance));
    assert_eq!(MyResolver::endpoint("admin@instance.tld", "admin"), Err(ResolverError::InvalidResource));
    assert_eq!(MyResolver::endpoint("admin", "admin"), Err(ResolverError::InvalidResource));
    assert_eq!(MyResolver::endpoint("acct:admin", "admin"), Err(ResolverError::InvalidResource));
}
