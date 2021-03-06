//! Data in memory layout of objects.
pub use std::alloc::LayoutErr;

use crate::ptr::NonNull;
use std::alloc::Layout as Inner;
use std::mem::{align_of, size_of};

/// Specialized [`Result`](std::result::Result) for
/// [`Layout`] operations.
pub type Result<T> = std::result::Result<T, LayoutErr>;

/// Get `(size, align)` for [`Sized`] types.
#[allow(clippy::inline_always)]
#[must_use]
#[inline(always)] // used only once
const fn size_align<T>() -> (usize, usize) {
    (size_of::<T>(), align_of::<T>())
}

/// Get `(size, align)` from reference.
#[allow(clippy::inline_always)]
#[must_use]
#[inline(always)] // used only once
const fn size_align_val<T: ?Sized>(val: &T) -> (usize, usize) {
    use std::mem::{align_of_val, size_of_val};
    (size_of_val(val), align_of_val(val))
}

/// Get `(size, align)` from raw pointer.
///
/// # Safety
///
/// This function is only safe to call if the following conditions hold:
///
/// - If `T` is `Sized`, this function is always safe to call.
/// - If the unsized tail of `T` is:
///     - a [slice](std::slice), then the length of the slice tail must be an
///       intialized integer, and the size of the *entire value* (dynamic tail
///       length + statically sized prefix) must fit in `isize`.
///     - a *trait object*, then the vtable part of the pointer must point to a
///       valid vtable for the type `T` acquired by an unsizing coersion, and
///       the size of the *entire value* (dynamic tail length + statically sized
///       prefix) must fit in `isize`.
///     - an (unstable) extern type, then this function is always safe to call,
///       but may panic or otherwise return the wrong value, as the extern
///       type's layout is not known. This is the same behavior as
///       [`Layout::for_value`] on a reference to an extern type tail.
///     - otherwise, it is conservatively not allowed to call this function.
///
/// See [`std::mem::size_of_val_raw`] and [`std::mem::align_of_val_raw`].
#[allow(clippy::inline_always)]
#[must_use]
#[inline(always)] // used only once
const unsafe fn size_align_val_raw<T: ?Sized>(ptr: *const T) -> (usize, usize) {
    use std::intrinsics::{min_align_of_val, size_of_val};
    (size_of_val(ptr), min_align_of_val(ptr))
}

/// Maximum of two `usize`s.
#[allow(clippy::inline_always)]
#[must_use]
#[inline(always)] // used only once
const fn max(a: usize, b: usize) -> usize {
    if hint::likely!(a >= b) {
        a
    } else {
        b
    }
}

/// Checks if type with `size` will overflow when
/// padded to `align`.
#[allow(clippy::inline_always)]
#[must_use]
#[inline(always)] // used only once
const fn overflow_padded(size: usize, align: usize) -> bool {
    // From: std::alloc::Layout::from_size_align

    // Rounded up size is:
    //   size_rounded_up = (size + align - 1) & !(align - 1);
    //
    // We know from above that align != 0. If adding (align - 1)
    // does not overflow, then rounding up will be fine.
    //
    // Conversely, &-masking with !(align - 1) will subtract off
    // only low-order-bits. Thus if overflow occurs with the sum,
    // the &-mask cannot subtract enough to undo that overflow.
    //
    // Above implies that checking for summation overflow is both
    // necessary and sufficient.
    size > usize::MAX - align.wrapping_sub(1)
}

/// Instance of [`LayoutErr`].
pub const LAYOUT_ERR: LayoutErr = match Inner::from_size_align(0, 0) {
    // check that the error is a unitary type
    Err(err) if size_of::<LayoutErr>() == 0 => err,
    _ => unreachable!(),
};

/// Layout of a block of memory.
///
/// An instance of `Layout` describes a particular layout of memory.
/// You build a `Layout` up as an input to give to an allocator.
///
/// All layouts have an associated size and a power-of-two alignment.
///
/// This is a wrapper for [`std::alloc::Layout`], making most
/// methods `const`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Layout(pub(crate) std::alloc::Layout);

impl Layout {
    /// Constructs a `Layout` from a given `size` and `align`.
    ///
    /// # Errors
    ///
    /// Returns `LayoutErr` if any of the following conditions are not met:
    ///
    /// * `align` must not be zero,
    ///
    /// * `align` must be a power of two,
    ///
    /// * `size`, when rounded up to the nearest multiple of `align`, must not
    ///   overflow (i.e., the rounded value must be less than or equal to
    ///   `usize::MAX`).
    ///
    /// See [`std::alloc::Layout::from_size_align`].
    #[inline]
    pub const fn from_size_align(size: usize, align: usize) -> Result<Self> {
        // 0 is not a power of 2
        if hint::unlikely!(!align.is_power_of_two()) {
            return Err(LAYOUT_ERR)
        }
        // size_rounded_up = (size + align - 1) & !(align - 1);
        if hint::unlikely!(overflow_padded(size, align)) {
            return Err(LAYOUT_ERR)
        }

        // SAFETY: checks above
        Ok(unsafe { Self::from_size_align_unchecked(size, align) })
    }

    /// Creates a layout, bypassing all checks.
    ///
    /// See [`std::alloc::Layout::from_size_align_unchecked`].
    ///
    /// # Safety
    ///
    /// This function is unsafe as it does not verify the preconditions from
    /// [`Layout::from_size_align`].
    #[must_use]
    #[inline]
    pub const unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Self {
        // SAFETY: the caller must guarantee a valid layout
        Self(unsafe { Inner::from_size_align_unchecked(size, align) })
    }

    /// The minimum size in bytes for a memory block of this layout.
    ///
    /// See [`std::alloc::Layout::size`].
    #[allow(clippy::inline_always)]
    #[must_use]
    #[inline(always)] // carry assumption to caller
    pub const fn size(&self) -> usize {
        let size = self.0.size();
        // SAFTEY: a valid layout will never overflow when padded
        unsafe { hint::assume!(!overflow_padded(size, self.align())) };
        // this hint is for inlining
        size
    }

    /// The minimum byte alignment for a memory block of this layout.
    ///
    /// See [`std::alloc::Layout::align`].
    #[allow(clippy::inline_always)]
    #[must_use]
    #[inline(always)] // carry assumption to caller
    pub const fn align(&self) -> usize {
        let align = self.0.align();
        // SAFTEY: align has to be a power of two
        unsafe { hint::assume!(align.is_power_of_two()) };
        // this hint is for inlining
        align
    }

    /// Creates a `NonNull` that is dangling, but well-aligned for this Layout.
    ///
    /// Note that the pointer value may potentially represent a valid pointer,
    /// which means this must not be used as a "not yet initialized"
    /// sentinel value. Types that lazily allocate must track initialization by
    /// some other means.
    ///
    /// See [`std::alloc::Layout::dangling`].
    #[must_use]
    #[inline]
    pub const fn dangling(&self) -> NonNull<u8> {
        NonNull(self.0.dangling())
    }

    /// Constructs a `Layout` suitable for holding a value of type `T`.
    ///
    /// See [`std::alloc::Layout::new`].
    #[must_use]
    #[inline]
    pub const fn new<T>() -> Self {
        let (size, align) = size_align::<T>();
        // SAFETY: rust types garantees a valid layout
        unsafe { Self::from_size_align_unchecked(size, align) }
    }

    /// Produces layout describing a record that could be used to
    /// allocate backing structure for `T` (which could be a trait
    /// or other unsized type like a slice).
    ///
    /// See [`std::alloc::Layout::for_value`].
    #[must_use]
    #[inline]
    pub const fn for_value<T: ?Sized>(val: &T) -> Self {
        let (size, align) = size_align_val(val);
        debug_assert!(Self::from_size_align(size, align).is_ok());
        // SAFETY: rust types garantees a valid layout
        unsafe { Self::from_size_align_unchecked(size, align) }
    }

    /// Produces layout describing a record that could be used to
    /// allocate backing structure for `T` (which could be a trait
    /// or other unsized type like a slice).
    ///
    /// # Safety
    ///
    /// This function is only safe to call if the following conditions hold:
    ///
    /// - If `T` is `Sized`, this function is always safe to call.
    /// - If the unsized tail of `T` is:
    ///     - a [slice](std::slice), then the length of the slice tail must be
    ///       an intialized integer, and the size of the *entire value* (dynamic
    ///       tail length + statically sized prefix) must fit in `isize`.
    ///     - a *trait object*, then the vtable part of the pointer must point
    ///       to a valid vtable for the type `T` acquired by an unsizing
    ///       coersion, and the size of the *entire value* (dynamic tail length
    ///       + statically sized prefix) must fit in `isize`.
    ///     - an (unstable) extern type, then this function is always safe to
    ///       call, but may panic or otherwise return the wrong value, as the
    ///       extern type's layout is not known. This is the same behavior as
    ///       [`Layout::for_value`] on a reference to an extern type tail.
    ///     - otherwise, it is conservatively not allowed to call this function.
    ///
    /// See [`std::alloc::Layout::for_value_raw`].
    #[must_use]
    #[inline]
    pub const unsafe fn for_value_raw<T: ?Sized>(val: *const T) -> Self {
        // SAFETY: caller guarantees valid type
        let (size, align) = unsafe { size_align_val_raw(val) };
        debug_assert!(Self::from_size_align(size, align).is_ok());
        // SAFETY: given that the type is valid, tust guarantees a valid layout
        unsafe { Self::from_size_align_unchecked(size, align) }
    }

    /// Returns the amount of padding we must insert after `self`
    /// to ensure that the following address will satisfy `align`
    /// (measured in bytes).
    ///
    /// See [`std::alloc::Layout::padding_needed_for`].
    #[must_use]
    #[inline]
    pub const fn padding_needed_for(&self, align: usize) -> usize {
        let len = self.size();
        // same as: ceil(len / align) * align
        // SAFETY: layout guarantess that size cannot overflow when padded
        let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);

        // SAFETY: size when padded >= size
        len_rounded_up.wrapping_sub(len)
    }

    /// Creates a layout by rounding the size of this layout up to a multiple
    /// of the layout's alignment.
    ///
    /// See [`std::alloc::Layout::pad_to_align`].
    #[must_use]
    #[inline]
    pub const fn pad_to_align(&self) -> Self {
        let pad = self.padding_needed_for(self.align());
        // SAFETY: layout guarantess that size cannot overflow when padded
        let new_size = self.size().wrapping_add(pad);

        // SAFETY: guaranteed by `padding_needed_for`
        unsafe { Self::from_size_align_unchecked(new_size, self.align()) }
    }

    /// Creates a layout describing the record for `self` followed by
    /// `next`, including any necessary padding to ensure that `next`
    /// will be properly aligned, but *no trailing padding*.
    ///
    /// See [`std::alloc::Layout::extend`].
    ///
    /// # Errors
    ///
    /// This will only error in case of arithmetic overflow or
    /// if it would overflow when padding.
    #[inline]
    pub const fn extend(&self, next: Self) -> Result<(Self, usize)> {
        let new_align = max(self.align(), next.align());
        let pad = self.padding_needed_for(next.align());

        let offset = match self.size().checked_add(pad) {
            Some(offset) => offset,
            None => return Err(LAYOUT_ERR),
        };
        let new_size = match offset.checked_add(next.size()) {
            Some(size) => size,
            None => return Err(LAYOUT_ERR),
        };

        // SAFETY: the old Layouts already checked for power of 2
        unsafe { hint::assume!(new_align.is_power_of_two()) };
        match Self::from_size_align(new_size, new_align) {
            Err(err) => Err(err),
            Ok(layout) => Ok((layout, offset)),
        }
    }

    /// Const version of [`PartialEq::eq`].
    #[must_use]
    #[inline]
    pub const fn eq(&self, other: &Self) -> bool {
        self.size() == other.size() && self.align() == other.align()
    }

    /// Layout for an empty, zero-sized `#[repr(C)]` struct.
    ///
    /// # Example
    ///
    /// ```
    /// # use mem::alloc::Layout;
    /// #[repr(C)]
    /// struct Empty {}
    ///
    /// assert_eq!(Layout::EMPTY, Layout::new::<Empty>());
    /// ```
    pub const EMPTY: Self = match Self::from_size_align(0, 1) {
        // check that the empty layout doesn't need padding
        Ok(layout) if layout.eq(&layout.pad_to_align()) => layout,
        _ => unreachable!(),
    };

    /// Repeatedly apply [`Layout::extend`].
    ///
    /// # Example
    ///
    /// The layout of a `#[repr(C)]` struct:
    ///
    /// ```
    /// use mem::alloc::Layout;
    ///
    /// #[repr(C)]
    /// struct CStruct {
    ///     id: u32,
    ///     name: String,
    ///     rank: f64
    /// }
    ///
    /// let fields = [Layout::new::<u32>(), Layout::new::<String>(), Layout::new::<f64>()];
    /// let (unpadded, offsets) = Layout::EMPTY.extend_many(fields).unwrap();
    /// let layout = unpadded.pad_to_align();
    ///
    /// assert_eq!(layout, Layout::new::<CStruct>())
    /// ```
    ///
    /// # Errors
    ///
    /// This will only error in case of arithmetic overflow or
    /// if it would overflow when padding.
    #[inline]
    pub const fn extend_many<const N: usize>(
        &self,
        layouts: [Self; N],
    ) -> Result<(Self, [usize; N])> {
        let mut offsets = [0; N];
        let mut layout = *self;

        let mut i = 0;
        while hint::likely!(i < N) {
            let (new, offset) = match layout.extend(layouts[i]) {
                Ok(data) => data,
                Err(err) => return Err(err),
            };

            offsets[i] = offset;
            layout = new;

            i += 1;
        }

        Ok((layout, offsets))
    }

    /// Checks that a pointer is aligned for this layout.
    ///
    /// # Example
    ///
    /// ```
    /// use mem::alloc::Layout;
    ///
    /// let text = "some string";
    /// let layout = Layout::for_value(text);
    ///
    /// assert!(layout.is_aligned(text as *const str))
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_aligned<T: ?Sized>(&self, ptr: *const T) -> bool {
        // type requirement
        debug_assert!(self.align().is_power_of_two());

        // SAFETY: every pointer generated at const context
        // should mantain alignment information, inclusive at runtime
        let addr = unsafe { ptr as *const u8 as usize };
        // bitmask to get the first bits which are the remainder
        // dividing by align, given that align is a power of two
        let mask = self.align().wrapping_sub(1);
        // same as: addr % align == 0
        (addr & mask) == 0
    }

    /// Recover inner [`std::alloc::Layout`] from `Layout`.
    #[allow(clippy::inline_always)]
    #[must_use]
    #[inline(always)] // transparent transmormation
    pub const fn inner(self) -> std::alloc::Layout {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guarantees that `Layout` method equivalence
    #[test]
    fn equivalent_methods() {
        use std::alloc::Layout as Inner;
        use std::usize::MAX;

        type T1 = ();
        type T2 = String;
        let layout1 = Layout::new::<T1>();
        let layout2 = Layout::new::<T2>();

        assert_eq!(layout1.inner(), Inner::new::<T1>());
        assert_eq!(layout2.inner(), Inner::new::<T2>());

        assert_eq!(
            Layout::from_size_align(10, 4),
            Inner::from_size_align(10, 4).map(Layout)
        );
        assert_eq!(
            Layout::from_size_align(13, 7),
            Inner::from_size_align(13, 7).map(Layout)
        );
        assert_eq!(
            Layout::from_size_align(MAX, 16),
            Inner::from_size_align(MAX, 16).map(Layout)
        );

        unsafe {
            assert_eq!(
                // SAFETY: align is valid and can't overflow
                Layout::from_size_align_unchecked(24, 8),
                Layout(Inner::from_size_align_unchecked(24, 8))
            );
        }

        assert_eq!(layout1.align(), layout1.inner().align());
        assert_eq!(layout2.size(), layout2.inner().size());

        let val = "string";
        assert_eq!(Layout::for_value(val), Layout(Inner::for_value(val)));
        let ptr = val as *const str;
        unsafe {
            assert_eq!(
                // SAFETY: `ptr` is a valid reference
                Layout::for_value_raw(ptr),
                Layout(Inner::for_value_raw(ptr))
            );
        }

        assert_eq!(
            layout2.padding_needed_for(256),
            layout2.inner().padding_needed_for(256)
        );
        assert_eq!(
            layout1.pad_to_align().inner(),
            layout1.inner().pad_to_align()
        );

        assert_eq!(
            layout1.extend(layout2),
            layout1
                .inner()
                .extend(layout2.inner())
                .map(|(a, b)| (Layout(a), b))
        );
        let overflow = Layout::from_size_align(MAX - 4, 2).unwrap();
        assert_eq!(
            layout2.extend(overflow),
            layout2
                .inner()
                .extend(overflow.inner())
                .map(|(a, b)| (Layout(a), b))
        );
    }
}
