use serde_json;
use super::*;

#[test]
fn test_url_for() {
    assert_eq!(
        url_for(Prefix::Acct, "test@example.org", true),
        Ok(String::from("https://example.org/.well-known/webfinger?resource=acct:test@example.org"))
    );
    assert_eq!(
        url_for(Prefix::Acct, "test", true),
        Err(WebfingerError::ParseError)
    );
    assert_eq!(
        url_for(Prefix::Acct, "test@example.org", false),
        Ok(String::from("http://example.org/.well-known/webfinger?resource=acct:test@example.org"))
    );
    assert_eq!(
        url_for(Prefix::Group, "test@example.org", true),
        Ok(String::from("https://example.org/.well-known/webfinger?resource=group:test@example.org"))
    );
    assert_eq!(
        url_for(Prefix::Custom("hey".into()), "test@example.org", true),
        Ok(String::from("https://example.org/.well-known/webfinger?resource=hey:test@example.org"))
    );
}

#[test]
fn test_resolve() {
    let m = mockito::mock("GET", mockito::Matcher::Any)
        .with_body(r#"
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
            "#)
        .create();

    let url = format!("test@{}", mockito::server_url()).replace("http://", "");
    println!("{}", url);
    let res = resolve(url, false).unwrap();
    assert_eq!(res.subject, String::from("acct:test@example.org"));

    m.assert();
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
            href: Some("https://example.org/@test/".to_string()),
            template: None
        },
        Link {
            rel: "http://schemas.google.com/g/2010#updates-from".to_string(),
            mime_type: Some("application/atom+xml".to_string()),
            href: Some("https://example.org/@test/feed.atom".to_string()),
            template: None
        },
        Link {
            rel: "self".to_string(),
            mime_type: Some("application/activity+json".to_string()),
            href: Some("https://example.org/@test/".to_string()),
            template: None
        }
    ], webfinger.links);
}

pub struct MyResolver;

// Only one user, represented by a String
impl Resolver<&'static str> for MyResolver {
    fn instance_domain<'a>() -> &'a str {
        "instance.tld"
    }

    fn find(prefix: Prefix, acct: String, resource_repo: &'static str) -> Result<Webfinger, ResolverError> {
        if acct == resource_repo.to_string() && prefix == Prefix::Acct {
            Ok(Webfinger {
                subject: acct.clone(),
                aliases: vec![acct.clone()],
                links: vec![
                    Link {
                        rel: "http://webfinger.net/rel/profile-page".to_string(),
                        mime_type: None,
                        href: Some(format!("https://instance.tld/@{}/", acct)),
                        template: None
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
    assert_eq!(MyResolver::endpoint("acct:admin@oops.ie", "admin"), Err(ResolverError::WrongDomain));
    assert_eq!(MyResolver::endpoint("admin@instance.tld", "admin"), Err(ResolverError::InvalidResource));
    assert_eq!(MyResolver::endpoint("admin", "admin"), Err(ResolverError::InvalidResource));
    assert_eq!(MyResolver::endpoint("acct:admin", "admin"), Err(ResolverError::InvalidResource));
    assert_eq!(MyResolver::endpoint("group:admin@instance.tld", "admin"), Err(ResolverError::NotFound));
}
