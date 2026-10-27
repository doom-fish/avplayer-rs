#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::retained::retain_release_wrapper;
use crate::time::{Time, TimeRange};
use crate::util::{parse_value_and_free, to_cstring, value_buffer};

#[derive(Debug, Clone, PartialEq)]
struct MetadataGroupPayload {
    items: Vec<MetadataItem>,
    classifying_label: Option<String>,
    unique_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct TimedMetadataGroupPayload {
    items: Vec<MetadataItem>,
    classifying_label: Option<String>,
    unique_id: Option<String>,
    time_range: TimeRange,
}

#[derive(Debug, Clone, PartialEq)]
struct DateRangeMetadataGroupPayload {
    items: Vec<MetadataItem>,
    classifying_label: Option<String>,
    unique_id: Option<String>,
    start_date: String,
    end_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct MutableMetadataItemPayload {
    identifier: Option<String>,
    extended_language_tag: Option<String>,
    locale_identifier: Option<String>,
    time: Time,
    duration: Time,
    data_type: Option<String>,
    string_value: Option<String>,
    number_value: Option<f64>,
    value_description: Option<String>,
    start_date: Option<String>,
    key_space: Option<String>,
    key_string: Option<String>,
}

/// Borrowed `AVMetadataGroup` view.
#[derive(Debug, Clone, Copy)]
pub struct MetadataGroup<'a> {
    ptr: *mut c_void,
    _marker: PhantomData<&'a c_void>,
}

impl MetadataGroup<'_> {
    pub(crate) const fn from_ptr(ptr: *mut c_void) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    fn info(&self) -> Result<MetadataGroupPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe { ffi::av_metadata_group_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    pub fn items(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.items)
    }

    pub fn classifying_label(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.classifying_label)
    }

    pub fn unique_id(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.unique_id)
    }
}

/// Owned `AVTimedMetadataGroup` wrapper.
#[derive(Debug)]
pub struct TimedMetadataGroupHandle {
    ptr: *mut c_void,
}

retain_release_wrapper!(
    TimedMetadataGroupHandle,
    release = ffi::av_ns_object_release
);

impl TimedMetadataGroupHandle {
    pub(crate) const fn from_ptr(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub fn new(items: &[MetadataItem], time_range: TimeRange) -> Result<Self, AVPlayerError> {
        let items = value_buffer(items, "timed metadata items")?;
        let (start_value, start_timescale, start_kind) = time_range.start.to_raw();
        let (duration_value, duration_timescale, duration_kind) = time_range.duration.to_raw();
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_timed_metadata_group_create(
                items.as_ptr(),
                start_value,
                start_timescale,
                start_kind,
                duration_value,
                duration_timescale,
                duration_kind,
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<TimedMetadataGroupPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe { ffi::av_timed_metadata_group_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    pub const fn as_metadata_group(&self) -> MetadataGroup<'_> {
        MetadataGroup::from_ptr(self.ptr)
    }

    pub fn items(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.items)
    }

    pub fn classifying_label(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.classifying_label)
    }

    pub fn unique_id(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.unique_id)
    }

    pub fn time_range(&self) -> Result<TimeRange, AVPlayerError> {
        Ok(self.info()?.time_range)
    }
}

/// Owned `AVMutableTimedMetadataGroup` wrapper.
#[derive(Debug)]
pub struct MutableTimedMetadataGroup {
    ptr: *mut c_void,
}

retain_release_wrapper!(
    MutableTimedMetadataGroup,
    release = ffi::av_ns_object_release
);

impl MutableTimedMetadataGroup {
    pub fn new(items: &[MetadataItem], time_range: TimeRange) -> Result<Self, AVPlayerError> {
        let items = value_buffer(items, "mutable timed metadata items")?;
        let (start_value, start_timescale, start_kind) = time_range.start.to_raw();
        let (duration_value, duration_timescale, duration_kind) = time_range.duration.to_raw();
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_mutable_timed_metadata_group_create(
                items.as_ptr(),
                start_value,
                start_timescale,
                start_kind,
                duration_value,
                duration_timescale,
                duration_kind,
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<TimedMetadataGroupPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe { ffi::av_timed_metadata_group_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    pub const fn as_metadata_group(&self) -> MetadataGroup<'_> {
        MetadataGroup::from_ptr(self.ptr)
    }

    pub fn items(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.items)
    }

    pub fn classifying_label(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.classifying_label)
    }

    pub fn unique_id(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.unique_id)
    }

    pub fn time_range(&self) -> Result<TimeRange, AVPlayerError> {
        Ok(self.info()?.time_range)
    }

    pub fn set_time_range(&self, time_range: TimeRange) {
        let (start_value, start_timescale, start_kind) = time_range.start.to_raw();
        let (duration_value, duration_timescale, duration_kind) = time_range.duration.to_raw();
        unsafe {
            ffi::av_mutable_timed_metadata_group_set_time_range(
                self.ptr,
                start_value,
                start_timescale,
                start_kind,
                duration_value,
                duration_timescale,
                duration_kind,
            );
        };
    }

    pub fn set_items(&self, items: &[MetadataItem]) -> Result<(), AVPlayerError> {
        let items = value_buffer(items, "mutable timed metadata items")?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe {
            ffi::av_mutable_timed_metadata_group_set_items_value(
                self.ptr,
                items.as_ptr(),
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

/// Owned `AVDateRangeMetadataGroup` wrapper.
#[derive(Debug)]
pub struct DateRangeMetadataGroupHandle {
    ptr: *mut c_void,
}

retain_release_wrapper!(
    DateRangeMetadataGroupHandle,
    release = ffi::av_ns_object_release
);

impl DateRangeMetadataGroupHandle {
    pub fn new(
        items: &[MetadataItem],
        start_date: &str,
        end_date: Option<&str>,
    ) -> Result<Self, AVPlayerError> {
        let items = value_buffer(items, "date-range metadata items")?;
        let start_date = to_cstring(start_date, "date-range start date")?;
        let end_date = end_date
            .map(|value| to_cstring(value, "date-range end date"))
            .transpose()?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_date_range_metadata_group_create(
                items.as_ptr(),
                start_date.as_ptr(),
                end_date
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<DateRangeMetadataGroupPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr =
            unsafe { ffi::av_date_range_metadata_group_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    pub const fn as_metadata_group(&self) -> MetadataGroup<'_> {
        MetadataGroup::from_ptr(self.ptr)
    }

    pub fn items(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.items)
    }

    pub fn classifying_label(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.classifying_label)
    }

    pub fn unique_id(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.unique_id)
    }

    pub fn start_date(&self) -> Result<String, AVPlayerError> {
        Ok(self.info()?.start_date)
    }

    pub fn end_date(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.end_date)
    }
}

/// Owned `AVMutableDateRangeMetadataGroup` wrapper.
#[derive(Debug)]
pub struct MutableDateRangeMetadataGroup {
    ptr: *mut c_void,
}

retain_release_wrapper!(
    MutableDateRangeMetadataGroup,
    release = ffi::av_ns_object_release
);

impl MutableDateRangeMetadataGroup {
    pub fn new(
        items: &[MetadataItem],
        start_date: &str,
        end_date: Option<&str>,
    ) -> Result<Self, AVPlayerError> {
        let items = value_buffer(items, "mutable date-range metadata items")?;
        let start_date = to_cstring(start_date, "mutable date-range start date")?;
        let end_date = end_date
            .map(|value| to_cstring(value, "mutable date-range end date"))
            .transpose()?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_mutable_date_range_metadata_group_create(
                items.as_ptr(),
                start_date.as_ptr(),
                end_date
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn into_ptr(mut self) -> *mut c_void {
        let ptr = self.ptr;
        self.ptr = ptr::null_mut();
        ptr
    }

    fn replace_group(
        &mut self,
        items: &[MetadataItem],
        start_date: &str,
        end_date: Option<&str>,
    ) -> Result<(), AVPlayerError> {
        // AVMutableDateRangeMetadataGroup's property setters crash on current macOS SDKs,
        // so rebuild the object through its initializer and swap the retained handle.
        let replacement = Self::new(items, start_date, end_date)?;
        let old_ptr = self.ptr;
        self.ptr = replacement.into_ptr();
        if !old_ptr.is_null() {
            unsafe { ffi::av_ns_object_release(old_ptr) };
        }
        Ok(())
    }

    fn info(&self) -> Result<DateRangeMetadataGroupPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr =
            unsafe { ffi::av_date_range_metadata_group_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    pub const fn as_metadata_group(&self) -> MetadataGroup<'_> {
        MetadataGroup::from_ptr(self.ptr)
    }

    pub fn items(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.items)
    }

    pub fn classifying_label(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.classifying_label)
    }

    pub fn unique_id(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.unique_id)
    }

    pub fn start_date(&self) -> Result<String, AVPlayerError> {
        Ok(self.info()?.start_date)
    }

    pub fn end_date(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.end_date)
    }

    pub fn set_start_date(&mut self, start_date: &str) -> Result<(), AVPlayerError> {
        let info = self.info()?;
        self.replace_group(&info.items, start_date, info.end_date.as_deref())
    }

    pub fn set_end_date(&mut self, end_date: Option<&str>) -> Result<(), AVPlayerError> {
        let info = self.info()?;
        self.replace_group(&info.items, &info.start_date, end_date)
    }

    pub fn set_items(&mut self, items: &[MetadataItem]) -> Result<(), AVPlayerError> {
        let info = self.info()?;
        self.replace_group(items, &info.start_date, info.end_date.as_deref())
    }
}

/// Owned `AVMutableMetadataItem` wrapper.
#[derive(Debug)]
pub struct MutableMetadataItem {
    ptr: *mut c_void,
}

retain_release_wrapper!(MutableMetadataItem, release = ffi::av_ns_object_release);

impl MutableMetadataItem {
    pub fn new() -> Result<Self, AVPlayerError> {
        let ptr = unsafe { ffi::av_mutable_metadata_item_create() };
        if ptr.is_null() {
            return Err(AVPlayerError::OperationFailed(
                "bridge returned null mutable metadata item".into(),
            ));
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<MutableMetadataItemPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe { ffi::av_mutable_metadata_item_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    pub fn identifier(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.identifier)
    }

    pub fn extended_language_tag(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.extended_language_tag)
    }

    pub fn locale_identifier(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.locale_identifier)
    }

    pub fn time(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.time)
    }

    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.duration)
    }

    pub fn data_type(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.data_type)
    }

    pub fn string_value(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.string_value)
    }

    pub fn number_value(&self) -> Result<Option<f64>, AVPlayerError> {
        Ok(self.info()?.number_value)
    }

    pub fn value_description(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.value_description)
    }

    pub fn start_date(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.start_date)
    }

    pub fn key_space(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.key_space)
    }

    pub fn key_string(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.key_string)
    }

    pub fn set_identifier(&self, identifier: Option<&str>) -> Result<(), AVPlayerError> {
        let identifier = identifier
            .map(|value| to_cstring(value, "metadata identifier"))
            .transpose()?;
        unsafe {
            ffi::av_mutable_metadata_item_set_identifier(
                self.ptr,
                identifier
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
            );
        };
        Ok(())
    }

    pub fn set_extended_language_tag(
        &self,
        language_tag: Option<&str>,
    ) -> Result<(), AVPlayerError> {
        let language_tag = language_tag
            .map(|value| to_cstring(value, "metadata language tag"))
            .transpose()?;
        unsafe {
            ffi::av_mutable_metadata_item_set_extended_language_tag(
                self.ptr,
                language_tag
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
            );
        };
        Ok(())
    }

    pub fn set_locale_identifier(
        &self,
        locale_identifier: Option<&str>,
    ) -> Result<(), AVPlayerError> {
        let locale_identifier = locale_identifier
            .map(|value| to_cstring(value, "metadata locale identifier"))
            .transpose()?;
        unsafe {
            ffi::av_mutable_metadata_item_set_locale_identifier(
                self.ptr,
                locale_identifier
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
            );
        };
        Ok(())
    }

    pub fn set_time(&self, time: Time) {
        let (value, timescale, kind) = time.to_raw();
        unsafe { ffi::av_mutable_metadata_item_set_time(self.ptr, value, timescale, kind) };
    }

    pub fn set_duration(&self, duration: Time) {
        let (value, timescale, kind) = duration.to_raw();
        unsafe { ffi::av_mutable_metadata_item_set_duration(self.ptr, value, timescale, kind) };
    }

    pub fn set_data_type(&self, data_type: Option<&str>) -> Result<(), AVPlayerError> {
        let data_type = data_type
            .map(|value| to_cstring(value, "metadata data type"))
            .transpose()?;
        unsafe {
            ffi::av_mutable_metadata_item_set_data_type(
                self.ptr,
                data_type
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
            );
        };
        Ok(())
    }

    pub fn set_string_value(&self, value: Option<&str>) -> Result<(), AVPlayerError> {
        let value = value
            .map(|raw| to_cstring(raw, "metadata string value"))
            .transpose()?;
        unsafe {
            ffi::av_mutable_metadata_item_set_string_value(
                self.ptr,
                value
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
            );
        };
        Ok(())
    }

    pub fn set_number_value(&self, value: f64) {
        unsafe { ffi::av_mutable_metadata_item_set_number_value(self.ptr, value) };
    }

    pub fn clear_value(&self) {
        unsafe { ffi::av_mutable_metadata_item_clear_value(self.ptr) };
    }

    pub fn set_start_date(&self, start_date: Option<&str>) -> Result<(), AVPlayerError> {
        let start_date = start_date
            .map(|value| to_cstring(value, "metadata start date"))
            .transpose()?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe {
            ffi::av_mutable_metadata_item_set_start_date(
                self.ptr,
                start_date
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_key_space(&self, key_space: Option<&str>) -> Result<(), AVPlayerError> {
        let key_space = key_space
            .map(|value| to_cstring(value, "metadata key space"))
            .transpose()?;
        unsafe {
            ffi::av_mutable_metadata_item_set_key_space(
                self.ptr,
                key_space
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
            );
        };
        Ok(())
    }

    pub fn set_key_string(&self, key: Option<&str>) -> Result<(), AVPlayerError> {
        let key = key
            .map(|value| to_cstring(value, "metadata key"))
            .transpose()?;
        unsafe {
            ffi::av_mutable_metadata_item_set_key_string(
                self.ptr,
                key.as_ref()
                    .map_or(ptr::null(), super::binary::InputString::as_ptr),
            );
        };
        Ok(())
    }
}

/// Owned `AVMetadataItemFilter` wrapper.
#[derive(Debug)]
pub struct MetadataItemFilter {
    ptr: *mut c_void,
}

retain_release_wrapper!(MetadataItemFilter, release = ffi::av_ns_object_release);

impl MetadataItemFilter {
    pub fn for_sharing() -> Result<Self, AVPlayerError> {
        let ptr = unsafe { ffi::av_metadata_item_filter_create_for_sharing() };
        if ptr.is_null() {
            return Err(AVPlayerError::OperationFailed(
                "bridge returned null metadata item filter".into(),
            ));
        }
        Ok(Self { ptr })
    }

    pub fn filter(&self, items: &[MetadataItem]) -> Result<Vec<MetadataItem>, AVPlayerError> {
        let items = value_buffer(items, "metadata items to filter")?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe {
            ffi::av_metadata_item_filter_filter_value(self.ptr, items.as_ptr(), &raw mut err)
        };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }
}

// SAFETY: These Objective-C metadata handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for TimedMetadataGroupHandle {}
unsafe impl Send for MutableTimedMetadataGroup {}
unsafe impl Send for DateRangeMetadataGroupHandle {}
unsafe impl Send for MutableDateRangeMetadataGroup {}
unsafe impl Send for MutableMetadataItem {}
unsafe impl Send for MetadataItemFilter {}

impl crate::binary::FromValue for MetadataGroupPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            items: value.field(0)?,
            classifying_label: value.field(1)?,
            unique_id: value.field(2)?,
        })
    }
}

impl crate::binary::FromValue for TimedMetadataGroupPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            items: value.field(0)?,
            classifying_label: value.field(1)?,
            unique_id: value.field(2)?,
            time_range: value.field(3)?,
        })
    }
}

impl crate::binary::FromValue for DateRangeMetadataGroupPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            items: value.field(0)?,
            classifying_label: value.field(1)?,
            unique_id: value.field(2)?,
            start_date: value.field(3)?,
            end_date: value.field(4)?,
        })
    }
}

impl crate::binary::FromValue for MutableMetadataItemPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            identifier: value.field(0)?,
            extended_language_tag: value.field(1)?,
            locale_identifier: value.field(2)?,
            time: value.field(3)?,
            duration: value.field(4)?,
            data_type: value.field(5)?,
            string_value: value.field(6)?,
            number_value: value.field(7)?,
            value_description: value.field(8)?,
            start_date: value.field(9)?,
            key_space: value.field(10)?,
            key_string: value.field(11)?,
        })
    }
}
