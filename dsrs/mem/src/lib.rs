#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(const_ptr_is_null)]
#![feature(const_ptr_offset)]
#![feature(const_mut_refs)]
#![feature(const_raw_ptr_deref)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(const_size_of_val)]
#![feature(const_align_of_val)]
#![feature(const_alloc_layout)]
#![feature(const_result)]
#![feature(const_panic)]
#![feature(const_likely)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(dispatch_from_dyn)]
extern crate hint;

pub mod ptr;
pub mod layout;
