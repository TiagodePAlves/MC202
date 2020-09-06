use proc_macro2::Span;
use syn::parse::Error;
use std::ops::BitOr;

/// Specialized [`Result`](syn::parse::Result) for checking.
///
/// This represents a computation, without result, that may encounter errors.
type Result = syn::parse::Result<()>;

/// Status marker for `#[repr(C)]` attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    /// No `C` hint found yet.
    Missing,
    /// Found at least one `#[repr(C)]` attribute or equivalent.
    Found
}
use Status::*;

impl Status {
    /// Returns [`Found`] when at least one of them is marked so.
    #[inline]
    pub const fn or(self, other: Self) -> Self {
        match (self, other) {
            // or missing when both are as well
            (Missing, Missing) => Missing,
            (_, _) => Found
        }
    }

    /// Turns a [`Missing`] into an [`Error`] at call site.
    ///
    /// `Found`s are `Ok`.
    #[inline]
    pub fn into_result(self) ->  Result {
        match self {
            Found => Ok(()),
            Missing => Err(Self::error())
        }
    }

    /// [`Error`] message when `#[repr(C)]` attribute is missing.
    const MESSAGE: &'static str = concat!(
        // this is constant because it is too large to well be indented
        "missing '#[repr(C)]' attribute, ",
        "'ReprC' trait cannot be safely implemented for other layouts"
    );

    /// Default error for when `C` hint is missing.
    /// Generated at [call site](Span::call_site).
    #[inline]
    fn error() -> Error {
        Error::new(Span::call_site(), Self::MESSAGE)
    }
}

impl Default for Status {
    #[inline]
    fn default() -> Self {
        Missing
    }
}

impl BitOr for Status {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        self.or(other)
    }
}

impl Into<Result> for Status {
    #[inline]
    fn into(self) -> Result {
        self.into_result()
    }
}
