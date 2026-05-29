//! Declarative macro for retain/release wrapper boilerplate.
//!
//! Many wrapper types hold a single raw pointer to a retained Objective-C /
//! Swift object and hand-roll identical `Drop` implementations that null-check
//! the pointer, call the matching `release` FFI function, and then clear the
//! field. `retain_release_wrapper!` consolidates that boilerplate into a single
//! audited place.
//!
//! The generated `Drop` impl preserves the exact behavior of the previous
//! hand-written versions:
//! - null-checks the pointer before calling the supplied `release` FFI fn
//!   (matching the original `if !ptr.is_null()` guards), and
//! - resets the field to `null_mut()` after releasing.
//!
//! Types whose `Drop` carries extra logic beyond null-check + release + reset
//! are intentionally left hand-written.

/// Generate a `Drop` impl for a retain/release pointer wrapper.
///
/// Variants:
/// - Default `ptr` field:
///   `retain_release_wrapper!(Ty, release = path::release);`
/// - Custom field name (e.g. `token`):
///   `retain_release_wrapper!(Ty, field = token, release = path::release);`
macro_rules! retain_release_wrapper {
    ($ty:ty, field = $field:ident, release = $release:path $(,)?) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.$field.is_null() {
                    unsafe { $release(self.$field) };
                    self.$field = ::core::ptr::null_mut();
                }
            }
        }
    };

    ($ty:ty, release = $release:path $(,)?) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.ptr.is_null() {
                    unsafe { $release(self.ptr) };
                    self.ptr = ::core::ptr::null_mut();
                }
            }
        }
    };
}

pub(crate) use retain_release_wrapper;
