#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::binary::InputString;
use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::fmt;
use core::ptr;

use crate::binary::FromValue;
use crate::binary::ToValue;
use doom_fish_utils::callback_context::CallbackContext;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::time::Time;

pub type Handler<E> = Box<dyn Fn(E) + Send + Sync + 'static>;
pub type BoolHandler<E> = Box<dyn Fn(E) -> bool + Send + Sync + 'static>;

trait ContextHandle {
    fn deactivate(&self);
}

impl<T: Send + Sync + 'static> ContextHandle for CallbackContext<T> {
    fn deactivate(&self) {
        Self::deactivate(self);
    }
}

pub struct Registration {
    token: *mut c_void,
    release: unsafe extern "C" fn(*mut c_void),
    context: Box<dyn ContextHandle + Send + Sync>,
}

unsafe impl Send for Registration {}

impl Registration {
    pub fn new<T, F>(
        value: T,
        release: unsafe extern "C" fn(*mut c_void),
        register: F,
    ) -> Result<Self, AVPlayerError>
    where
        T: Send + Sync + 'static,
        F: FnOnce(*mut c_void, Option<ffi::DropCallback>, *mut *mut AVPValue) -> *mut c_void,
    {
        let context = CallbackContext::new(value);
        let mut err: *mut AVPValue = ptr::null_mut();
        let token = register(
            context.retained_ptr(),
            Some(CallbackContext::<T>::RELEASE),
            &raw mut err,
        );
        if token.is_null() {
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(Self {
            token,
            release,
            context: Box::new(context),
        })
    }
}

impl Registration {
    pub const fn as_ptr(&self) -> *mut c_void {
        self.token
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        self.context.deactivate();
        if !self.token.is_null() {
            unsafe { (self.release)(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

impl fmt::Debug for Registration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registration")
            .field("token", &self.token)
            .finish_non_exhaustive()
    }
}

pub unsafe fn deliver<E: 'static>(userdata: *mut c_void, site: &str, event: E) {
    let _ =
        unsafe { CallbackContext::<Handler<E>>::with(userdata, site, |handler| handler(event)) };
}

pub unsafe fn deliver_bool<E: 'static>(userdata: *mut c_void, site: &str, event: E) -> bool {
    unsafe { CallbackContext::<BoolHandler<E>>::with(userdata, site, |handler| handler(event)) }
        .unwrap_or(false)
}

pub unsafe fn value_payload<T: FromValue>(payload_value: *const AVPValue) -> Option<T> {
    let value = unsafe { crate::binary::read_value(payload_value) }.ok()?;
    T::from_value(&value).ok()
}

pub unsafe fn borrowed_objects<T>(
    objects: *const *mut c_void,
    count: usize,
    adopt: impl Fn(*mut c_void) -> Option<T>,
) -> Vec<T> {
    if objects.is_null() || count == 0 {
        return Vec::new();
    }
    unsafe { core::slice::from_raw_parts(objects, count) }
        .iter()
        .filter_map(|&object| adopt(object))
        .collect()
}

pub unsafe fn take_object_array(array: *mut c_void) -> Vec<*mut c_void> {
    if array.is_null() {
        return Vec::new();
    }
    let count = unsafe { ffi::avp_object_array_count(array) };
    let objects = (0..count)
        .map(|index| unsafe { ffi::avp_object_array_copy_at(array, index) })
        .collect();
    unsafe { ffi::avp_object_array_release(array) };
    objects
}

pub fn validate_seek_time(time: Time, what: &str) -> Result<(), AVPlayerError> {
    match time {
        Time::Numeric { timescale, .. } if timescale > 0 => Ok(()),
        Time::Numeric { .. } => Err(AVPlayerError::InvalidArgument(format!(
            "{what} needs a positive timescale"
        ))),
        Time::PositiveInfinity | Time::NegativeInfinity => Ok(()),
        Time::Invalid => Err(AVPlayerError::InvalidArgument(format!(
            "{what} must be a valid time"
        ))),
        Time::Indefinite => Err(AVPlayerError::InvalidArgument(format!(
            "{what} must not be indefinite"
        ))),
    }
}

pub fn validate_tolerance(tolerance: Time, what: &str) -> Result<(), AVPlayerError> {
    match tolerance {
        Time::Numeric { value, timescale } if timescale > 0 && value >= 0 => Ok(()),
        Time::Numeric { timescale, .. } if timescale > 0 => Err(AVPlayerError::InvalidArgument(
            format!("{what} must not be negative"),
        )),
        Time::PositiveInfinity => Ok(()),
        Time::NegativeInfinity => Err(AVPlayerError::InvalidArgument(format!(
            "{what} must not be negative"
        ))),
        Time::Numeric { .. } | Time::Invalid | Time::Indefinite => Err(
            AVPlayerError::InvalidArgument(format!("{what} must be a valid time")),
        ),
    }
}

pub fn validate_capacity(capacity: usize) -> Result<(), AVPlayerError> {
    if capacity == 0 {
        return Err(AVPlayerError::InvalidArgument(
            "event stream capacity must be at least 1".into(),
        ));
    }
    Ok(())
}

/// Calls the `AVPlayer` framework counterpart for `parse_value_and_free`.
pub fn parse_value_and_free<T: FromValue>(value_ptr: *mut AVPValue) -> Result<T, AVPlayerError> {
    let value = unsafe { crate::binary::take_value(value_ptr)? };
    T::from_value(&value)
}

/// Calls the `AVPlayer` framework counterpart for `to_cstring`.
pub fn to_cstring(value: &str, what: &str) -> Result<InputString, AVPlayerError> {
    InputString::new(value).map_err(|error| {
        AVPlayerError::InvalidArgument(format!("{what} contains NUL byte: {error}"))
    })
}

/// Calls the `AVPlayer` framework counterpart for `value_buffer`.
pub fn value_buffer<T: ToValue + ?Sized>(
    value: &T,
    what: &str,
) -> Result<crate::binary::InputValue, AVPlayerError> {
    crate::binary::InputValue::new(value)
        .map_err(|error| AVPlayerError::InvalidArgument(format!("invalid {what}: {error}")))
}

/// Calls the `AVPlayer` framework counterpart for `maybe_value_buffer`.
pub fn maybe_value_buffer<T: ToValue>(
    value: Option<&T>,
    what: &str,
) -> Result<Option<crate::binary::InputValue>, AVPlayerError> {
    value.map(|value| value_buffer(value, what)).transpose()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::*;

    struct DropCounter(Arc<AtomicUsize>);

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    unsafe extern "C" fn ignore_release(_token: *mut c_void) {}

    #[test]
    fn failed_registration_releases_the_foreign_reference() {
        let drops = Arc::new(AtomicUsize::new(0));
        let counter = DropCounter(Arc::clone(&drops));
        let handler: Handler<u32> = Box::new(move |_| {
            let _ = &counter;
        });
        let result = Registration::new(handler, ignore_release, |userdata, drop_userdata, _| {
            if let Some(drop_userdata) = drop_userdata {
                unsafe { drop_userdata(userdata) };
            }
            ptr::null_mut()
        });
        assert!(matches!(result, Err(AVPlayerError::ObserverFailed(_))));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn dropping_a_registration_deactivates_before_the_foreign_release() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&received);
        let handler: Handler<u32> = Box::new(move |value| sink.lock().unwrap().push(value));
        let mut foreign = (ptr::null_mut(), None);
        let registration =
            Registration::new(handler, ignore_release, |userdata, drop_userdata, _| {
                foreign = (userdata, drop_userdata);
                NonNullToken::VALUE
            })
            .unwrap();
        let (userdata, drop_userdata) = foreign;

        unsafe { deliver(userdata, "test", 1_u32) };
        drop(registration);
        unsafe { deliver(userdata, "test", 2_u32) };
        assert_eq!(*received.lock().unwrap(), vec![1]);
        assert_eq!(Arc::strong_count(&received), 2);

        unsafe { drop_userdata.unwrap()(userdata) };
        assert_eq!(Arc::strong_count(&received), 1);
    }

    struct NonNullToken;

    impl NonNullToken {
        const VALUE: *mut c_void = ptr::NonNull::<c_void>::dangling().as_ptr();
    }

    #[test]
    fn deliver_bool_defaults_to_false_for_inactive_contexts() {
        let handler: BoolHandler<u32> = Box::new(|value| value > 1);
        let context = CallbackContext::new(handler);
        assert!(unsafe { deliver_bool(context.as_ptr(), "test", 2_u32) });
        assert!(!unsafe { deliver_bool(context.as_ptr(), "test", 0_u32) });
        context.deactivate();
        assert!(!unsafe { deliver_bool(context.as_ptr(), "test", 2_u32) });
        assert!(!unsafe { deliver_bool(ptr::null_mut(), "test", 2_u32) });
    }

    #[test]
    fn seek_times_must_be_valid_and_definite() {
        assert!(validate_seek_time(Time::new(3, 600), "seek").is_ok());
        assert!(validate_seek_time(Time::positive_infinity(), "seek").is_ok());
        for time in [
            Time::invalid(),
            Time::indefinite(),
            Time::new(1, 0),
            Time::new(1, -1),
        ] {
            assert!(matches!(
                validate_seek_time(time, "seek"),
                Err(AVPlayerError::InvalidArgument(_))
            ));
        }
    }

    #[test]
    fn tolerances_must_be_valid_and_non_negative() {
        assert!(validate_tolerance(Time::new(0, 1), "tolerance").is_ok());
        assert!(validate_tolerance(Time::positive_infinity(), "tolerance").is_ok());
        for time in [
            Time::new(-1, 600),
            Time::negative_infinity(),
            Time::invalid(),
            Time::indefinite(),
            Time::new(1, 0),
        ] {
            assert!(matches!(
                validate_tolerance(time, "tolerance"),
                Err(AVPlayerError::InvalidArgument(_))
            ));
        }
    }

    #[test]
    fn zero_capacity_streams_are_rejected() {
        assert!(matches!(
            validate_capacity(0),
            Err(AVPlayerError::InvalidArgument(_))
        ));
        assert!(validate_capacity(1).is_ok());
    }

    #[test]
    fn borrowed_objects_skips_missing_entries() {
        let objects = [ptr::null_mut(), NonNullToken::VALUE];
        let adopted = unsafe {
            borrowed_objects(objects.as_ptr(), objects.len(), |object| {
                (!object.is_null()).then_some(object as usize)
            })
        };
        assert_eq!(adopted, vec![NonNullToken::VALUE as usize]);
        assert_eq!(
            unsafe { borrowed_objects(ptr::null(), 3, |_| Some(0_u8)) },
            [] as [u8; 0]
        );
    }
}
