use std::fmt::{Debug, Pointer, Formatter, Result};


/// Wrapper for [`std::ptr::NonNull`].
///
/// This wrapper makes [`from`](NonNull::from), [`as_ref`](NonNull::as_ref)
/// and [`as_mut`](NonNull::as_mut) as `const`.
#[repr(transparent)]
pub struct NonNull<T: ?Sized>(pub std::ptr::NonNull<T>);


impl<T: ?Sized> NonNull<T> {
    /// Read [`std::ptr::NonNull::new_unchecked`]
    #[inline]
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self(std::ptr::NonNull::new_unchecked(ptr))
    }

    /// Read [`std::ptr::NonNull::new`]
    #[inline]
    pub const fn new(ptr: *mut T) -> Option<Self> {
        if !ptr.is_null() {
            // SAFETY: already checked for null
            Some(unsafe { Self::new_unchecked(ptr) })
        } else {
            None
        }
    }

    /// Read [`std::ptr::NonNull::as_ptr`]
    #[inline]
    pub const fn as_ptr(self) -> *mut T {
        self.0.as_ptr()
    }

    /// Read [`std::ptr::NonNull::as_ref`]
    #[inline]
    pub const unsafe fn as_ref(&self) -> &T {
        &*(self.0.as_ptr() as *const T)
    }

    /// Read [`std::ptr::NonNull::as_mut`]
    #[inline]
    pub const unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.0.as_ptr()
    }

    /// Read [`std::ptr::NonNull::cast`]
    #[inline]
    pub const fn cast<U>(self) -> NonNull<U> {
        NonNull(self.0.cast())
    }

    /// Creates a new `NonNull` from a reference.
    ///
    /// Since a valid reference is never null, this is always safe.
    /// This is also conceptually equivalent to `value as
    /// *const T *mut T`, which is safe and guaranteed to be non null.
    ///
    /// Note: implemented as a `const` method intead of implementing
    /// the trait [`From`].
    #[inline]
    pub const fn from(value: &T) -> Self {
        let ptr = value as *const T as *mut T;
        // SAFETY: a reference is never null
        unsafe { Self::new_unchecked(ptr) }
    }

    /// Recover inner [`std::ptr::NonNull`] from `NonNull`.
    #[inline(always)]
    pub const fn inner(self) -> std::ptr::NonNull<T> {
        self.0
    }

    /// This is true when `NonNull<T>` is a fat pointer.
    #[inline]
    pub const fn is_fat_pointer() -> bool {
        use std::mem::size_of;

        /// Size of a fat pointer.
        const FAT_POINTER_SIZE: usize = size_of::<*mut [u8]>();
        debug_assert!(FAT_POINTER_SIZE != size_of::<*mut u8>());

        size_of::<Self>() == FAT_POINTER_SIZE
    }

    /// Split fat pointer into actual pointer and metadata.
    ///
    /// # Example
    ///
    /// ```
    /// # use dsrs::mem::NonNull;
    /// #
    /// let data = [1, 2, 3, 4];
    /// let ptr = NonNull::<[u32]>::from(&data);
    ///
    /// let (data_ptr, metadata) = ptr.split().unwrap();
    ///
    /// // metada for a slice is the length
    /// assert_eq!(metadata as usize, data.len());
    /// assert_eq!(NonNull::new(data_ptr), Some(ptr.cast()));
    /// ```
    #[inline]
    pub const fn split(&self) -> Option<(*mut u8, *mut u8)> {
        // SAFETY: no mutation happens, only reads
        let this = unsafe { &mut *(self as *const Self as *mut Self) };

        match this.split_mut() {
            Some((data, metadata)) => Some((*data, *metadata)),
            None => None
        }
    }

    /// Split fat pointer into actual pointer and metadata. Both
    /// are returned as mutable references to the parts of the
    /// fat pointer, so they can actually change the pointer
    /// inside the `NonNull` object.
    ///
    /// # Example
    ///
    /// ```
    /// # use dsrs::mem::NonNull;
    /// #
    /// let data = [1, 2, 3, 4];
    /// let mut ptr = NonNull::<[u32]>::from(&data);
    ///
    /// let (data_ptr, metadata) = ptr.split_mut().unwrap();
    ///
    /// // the metada for a slice is its length
    /// assert_eq!(*metadata as usize, data.len());
    ///
    /// // the references can actually mutate the NonNull
    /// *data_ptr = 0x1 as *mut u8;
    /// assert_eq!(ptr.cast().as_ptr(), 0x1 as *mut u8);
    /// ```
    #[inline]
    pub const fn split_mut(&mut self) -> Option<(&mut *mut u8, &mut *mut u8)> {
        if Self::is_fat_pointer() {
            // SAFETY: `inner` is a fat pointer
            Some(unsafe { self.split_mut_unchecked() })
        } else {
            None
        }
    }

    /// Split fat pointer into actual pointer and metadata. Both
    /// are returned as mutable references to the parts of the
    /// fat pointer, so they can actually change the pointer
    /// inside the `NonNull` object.
    ///
    /// See [`split_mut`](NonNull::split_mut) for examples.
    ///
    /// # Safety
    ///
    /// This is safe only if `*mut T` is a fat pointer. Otherwise,
    /// the metada reference is invalid.
    #[inline]
    pub const unsafe fn split_mut_unchecked(&mut self) -> (&mut *mut u8, &mut *mut u8) {
        let addr = self as *const Self as *mut *mut u8;
        (&mut *addr, &mut *addr.offset(1))
    }
}

impl<T: ?Sized> Clone for NonNull<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0)
    }

    #[inline(always)]
    fn clone_from(&mut self, other: &Self) {
        self.0 = other.0
    }
}
impl<T: ?Sized> Copy for NonNull<T> {}

impl<T: ?Sized> PartialEq for NonNull<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: ?Sized> Eq for NonNull<T> {}

impl<T: ?Sized> Debug for NonNull<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&self.inner(), f)
    }
}

impl<T: ?Sized> Pointer for NonNull<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Pointer::fmt(&self.inner(), f)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Guarantee that `*mut T`, `NonNull<T>` and `Option<NonNull<T>>`
    /// all have the same size, independently if `T` is an
    /// [Exotically Sized Type](https://doc.rust-lang.org/nomicon/exotic-sizes.html)
    #[test]
    fn packed_option_size() {
        use std::fmt::Debug;
        use std::mem::size_of;

        enum Void {}

        fn assert_size<T: ?Sized>(spec: &str) {
            eprintln!("Packed option: {}", spec);

            assert_eq!(size_of::<*mut T>(), size_of::<Option<NonNull<T>>>());
            assert_eq!(size_of::<NonNull<T>>(), size_of::<Option<NonNull<T>>>())
        }

        assert_size::<u32>("normal type");
        assert_size::<()>("zero sized type");
        assert_size::<dyn Debug>("trait object");
        assert_size::<str>("slice type");
        assert_size::<Void>("empty type")
    }

    /// Guarantees that `NonNull` methods are equivalent
    #[test]
    fn equivalent_methods() {
        use std::ptr::NonNull as Inner;

        let val = "string";
        let ptr = NonNull::from(val);

        assert_eq!(ptr.inner(), Inner::from(val));
        // SAFETY: `ptr` is a valid reference
        unsafe { assert_eq!(ptr.as_ref(), ptr.inner().as_ref()) };
        // SAFETY: `val` is not being used, so it can be mutable,
        // also `str` will not be mutated
        unsafe { assert_eq!(ptr.clone().as_mut(), ptr.inner().as_mut()) };

        let ptr = val as *const str as *mut str;
        assert_eq!(NonNull::new(ptr), Inner::new(ptr).map(NonNull));
        let null = std::ptr::null_mut::<i32>();
        assert_eq!(NonNull::new(null), Inner::new(null).map(NonNull));
    }
}
