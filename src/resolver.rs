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
        acct: String,
        resource_repo: R,
    ) -> Result<Webfinger, ResolverError>;

    /// Returns a WebFinger result for a requested resource.
    fn endpoint(
        &self,
        resource: impl Into<String>,
        resource_repo: R,
    ) -> Result<Webfinger, ResolverError> {
        let resource = resource.into();
        let mut parsed_query = resource.splitn(2, ':');
        let res_prefix = Prefix::from(parsed_query.next().ok_or(ResolverError::InvalidResource)?);
        let res = parsed_query.next().ok_or(ResolverError::InvalidResource)?;

        let mut parsed_res = res.splitn(2, '@');
        let user = parsed_res.next().ok_or(ResolverError::InvalidResource)?;
        let domain = parsed_res.next().ok_or(ResolverError::InvalidResource)?;
        if domain == self.instance_domain() {
            self.find(res_prefix, user.to_string(), resource_repo)
        } else {
            Err(ResolverError::WrongDomain)
        }
    }
}
