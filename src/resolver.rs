use crate::{Prefix, ResolverError, Webfinger};

/// A trait to easily generate a WebFinger endpoint for any resource repository.
///
/// The `R` type is your resource repository (a database for instance) that will be passed to the
/// [`find`](Resolver::find) and [`endpoint`](Resolver::endpoint) functions.
pub trait Resolver<R> {
    /// Returns the domain name of the current instance.
    fn instance_domain(&self) -> &str;

    /// Tries to find a resource, `acct`, in the repository `resource_repo`.
    ///
    /// `acct` is not a complete `acct:` URI, it only contains the identifier of the requested resource
    /// (e.g. `test` for `acct:test@example.org`)
    ///
    /// If the resource couldn't be found, you may probably want to return a [`ResolverError::NotFound`].
    fn find(
        &self,
        prefix: Prefix,
        acct: &str,
        rels: &[impl AsRef<str>],
        resource_repo: R,
    ) -> Result<Webfinger, ResolverError>;

    /// Returns a WebFinger result for a requested resource.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to resolve.
    /// * `rels` - The relations to resolve.
    ///     As described in the [RFC](https://www.rfc-editor.org/rfc/rfc7033#section-4.3),
    ///     there may be zero or more rel parameters, which can be used to restrict the
    ///     set of links returned to those that have the specified relation type.
    /// * `resource_repo` - The resource repository.
    fn endpoint(
        &self,
        resource: &str,
        rels: &[impl AsRef<str>],
        resource_repo: R,
    ) -> Result<Webfinger, ResolverError> {
        // Path for https://example.org/.well-known/webfinger/resource=acct:carol@example.com&rel=http://openid.net/specs/connect/1.0/issuer
        // resource = acct:carol@example.com
        // rel = http://openid.net/specs/connect/1.0/issuer
        let mut parsed_query = resource.splitn(2, ':');
        // parsed_query = ["acct", "carol@example.com"]
        let res_prefix = Prefix::from(parsed_query.next().ok_or(ResolverError::InvalidResource)?);
        // res_prefix = Prefix::Acct
        let res = parsed_query.next().ok_or(ResolverError::InvalidResource)?;
        // res = "carol@example.com"

        let mut parsed_res = res.splitn(2, '@');
        // parsed_res = ["carol", "example.com"]
        let user = parsed_res.next().ok_or(ResolverError::InvalidResource)?;
        // user = "carol"
        let domain = parsed_res.next().ok_or(ResolverError::InvalidResource)?;
        // domain = "example.com"
        if domain == self.instance_domain() {
            self.find(res_prefix, user, rels, resource_repo)
        } else {
            Err(ResolverError::WrongDomain)
        }
    }
}
