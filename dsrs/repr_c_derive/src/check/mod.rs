//! Parsing and checking for `#[repr(...)]` attributes.
mod hints;
mod attr;
mod status;

pub use hints::{ReprHint, ReprCHint};
pub use attr::AttrRepr;


/// Specialized [`Result`](syn::parse::Result) for checking.
///
/// This represents a computation, without result, that may encounter errors.
type Result = syn::parse::Result<()>;

/// Special [`and`][and] that [`combine`][err]s errors when both are `Err`.
///
/// [and]: std::result::Result::and
/// [err]: Error::combine
pub fn combine(earlier: Result, later: Result) -> Result {
    match (earlier, later) {
        (Ok(_), result) => result,
        (error, Ok(_)) => error,
        (Err(mut old), Err(new)) => {
            old.combine(new);
            Err(old)
        }
    }
}
