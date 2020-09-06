//! Checking validity of `#[repr(...)]` attributes.
use proc_macro2::Span;
use syn::parse::Error;
use std::convert::TryFrom;
use std::iter::FromIterator;

use super::{Result, combine, Status, ReprHint, ReprCHint};
use Status::{Missing, Found};

/// Result tracker for checking `#[repr(...)]` attributes.
#[derive(Debug, Clone)]
pub struct ReprResult {
    /// Current status for `#[repr(C)]` attribute or equivalent.
    repr_c: Status,
    /// Current result for other `#[repr(...)]` attributes.
    others: Result
}

impl ReprResult {
    /// Default result, considering no attributes checked yet.
    #[inline]
    const fn new() -> Self {
        Self {
            repr_c: Missing,
            others: Ok(())
        }
    }

    /// Result for when a attribute only marks as `#[repr(C)]`.
    #[inline]
    const fn found() -> Self {
        Self {
            repr_c: Found,
            others: Ok(())
        }
    }

    /// Result when another `#[repr(...)]` attribute is found at given `Span`.
    #[inline]
    fn other(at: Span) -> Self {
        Self {
            repr_c: Missing,
            others: Err(Self::error(at))
        }
    }

    /// Combines two results.
    ///
    /// Keeps track of errors from both results and if any of them found
    /// a valid `#[repr(C)]` attribute.
    #[inline]
    pub fn combine(self, other: Self) -> Self {
        Self {
            repr_c: self.repr_c | other.repr_c,
            others: combine(self.others, other.others)
        }
    }

    /// [`Error`] message when another `#[repr(...)]` attribute appears.
    const MESSAGE: &'static str = concat!(
        // constant because it is too large to be well indented
        "only '#[repr(C)]' structs can implement 'ReprC' trait safely, ",
        "with no other layout hints.", "

        ", "Read the documentation for the trait for a better explanation."
    );

    /// Default error for when another `repr` hint is found at given `Span`.
    fn error(at: Span) -> Error {
        Error::new(at, Self::MESSAGE)
    }
}


impl From<ReprCHint> for ReprResult {
    #[inline]
    fn from(_: ReprCHint) -> Self {
        Self::found()
    }
}

impl From<ReprHint> for ReprResult {
    #[inline]
    fn from(hint: ReprHint) -> Self {
        let err_loc = hint.span();
        match ReprCHint::try_from(hint) {
            Ok(c_hint) => Self::from(c_hint),
            Err(_) => Self::other(err_loc)
        }
    }
}

impl FromIterator<ReprResult> for ReprResult {
    #[inline]
    fn from_iter<I: IntoIterator<Item=ReprResult>>(iter: I) -> Self {
        iter.into_iter()
            .fold(Self::new(), Self::combine)
    }
}

impl Into<Result> for ReprResult {
    #[inline]
    fn into(self) -> Result {
        combine(self.repr_c.into(), self.others)
    }
}
