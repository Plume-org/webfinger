# WebFinger ![Crates.io](https://img.shields.io/crates/v/webfinger.svg) ![Libraries.io dependency status for GitHub repo](https://img.shields.io/librariesio/github/Plume-org/webfinger.svg) ![Codecov](https://img.shields.io/codecov/c/github/Plume-org/webfinger.svg) [![Build Status](https://travis-ci.org/Plume-org/webfinger.svg?branch=master)](https://travis-ci.org/Plume-org/webfinger)

A crate to help you fetch and serve WebFinger resources.

## Examples

Fetching a resource:

```rust
use webfinger::resolve;

fn main() {
    let res = resolve("acct:test@example.org", true).expect("Error while fetching resource");

    println!("Places to get more informations about {}:", res.subject);
    for link in res.links.into_iter() {
        println!("- {}", link.href);
    }
}
```

Serving resources:

```rust
use webfinger::Resolver;

pub struct MyResolver;

impl Resolver<DatabaseConnection> for MyResolver {
    fn instance_domain<'a>() -> &'a str {
        "instance.tld"
    }

    fn find(acct: String, db: DatabaseConnection) -> Result<Webfinger, ResolverError> {
        if let Some(user) = db.find_user_by_name(acct) {
            Ok(Webfinger {
                subject: acct.clone(),
                aliases: vec![acct.clone()],
                links: vec![
                    Link {
                        rel: "http://webfinger.net/rel/profile-page".to_string(),
                        mime_type: None,
                        href: user.profile_url()
                    }
                ]
            })
        } else {
            Err(ResolverError::NotFound)
        }
    }
}

fn main() {
    // Start a web server and map /.well-known/webfinger to a function calling MyResolver::endpoint
}
```
