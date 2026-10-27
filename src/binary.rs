#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::c_void;
use core::ptr;
use std::collections::BTreeMap;

use crate::error::AVPlayerError;
use crate::ffi::{self, AVPBytes, AVPValue};

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Real(f64),
    String(String),
    Data(Vec<u8>),
    Array(Vec<Self>),
    Dictionary(BTreeMap<String, Self>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Real(f64),
    String(String),
    Data(Vec<u8>),
    Array(Vec<Self>),
    Dictionary(BTreeMap<String, Self>),
    Object(usize),
}

pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError>;
}

pub trait ToValue {
    fn to_value(&self) -> Value;

    fn byte_buffer(_values: &[Self]) -> Option<Vec<u8>>
    where
        Self: Sized,
    {
        None
    }
}

pub fn invalid_value() -> AVPlayerError {
    AVPlayerError::OperationFailed("invalid binary bridge value".into())
}

impl Value {
    pub(crate) fn field<T: FromValue>(&self, index: usize) -> Result<T, AVPlayerError> {
        let Self::Array(fields) = self else {
            return Err(invalid_value());
        };
        T::from_value(fields.get(index).unwrap_or(&Self::Null))
    }

    pub(crate) fn field_or_default<T: FromValue + Default>(
        &self,
        index: usize,
    ) -> Result<T, AVPlayerError> {
        let Self::Array(fields) = self else {
            return Err(invalid_value());
        };
        match fields.get(index) {
            None | Some(Self::Null) => Ok(T::default()),
            Some(value) => T::from_value(value),
        }
    }
}

impl FromValue for Value {
    fn from_value(value: &Self) -> Result<Self, AVPlayerError> {
        Ok(value.clone())
    }
}

impl ToValue for Value {
    fn to_value(&self) -> Self {
        self.clone()
    }
}

impl FromValue for PropertyValue {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        Ok(match value {
            Value::Null => Self::Null,
            Value::Bool(value) => Self::Bool(*value),
            Value::Signed(value) => Self::Signed(*value),
            Value::Unsigned(value) => Self::Unsigned(*value),
            Value::Real(value) => Self::Real(*value),
            Value::String(value) => Self::String(value.clone()),
            Value::Data(value) => Self::Data(value.clone()),
            Value::Array(values) => Self::Array(
                values
                    .iter()
                    .map(Self::from_value)
                    .collect::<Result<_, _>>()?,
            ),
            Value::Dictionary(values) => Self::Dictionary(
                values
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), Self::from_value(value)?)))
                    .collect::<Result<_, AVPlayerError>>()?,
            ),
            Value::Object(_) => return Err(invalid_value()),
        })
    }
}

impl ToValue for PropertyValue {
    fn to_value(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(*value),
            Self::Signed(value) => Value::Signed(*value),
            Self::Unsigned(value) => Value::Unsigned(*value),
            Self::Real(value) => Value::Real(*value),
            Self::String(value) => Value::String(value.clone()),
            Self::Data(value) => Value::Data(value.clone()),
            Self::Array(values) => Value::Array(values.iter().map(Self::to_value).collect()),
            Self::Dictionary(values) => Value::Dictionary(
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_value()))
                    .collect(),
            ),
        }
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::String(value) => Ok(value.clone()),
            _ => Err(invalid_value()),
        }
    }
}

impl ToValue for str {
    fn to_value(&self) -> Value {
        Value::String(self.into())
    }
}

impl ToValue for &str {
    fn to_value(&self) -> Value {
        Value::String((*self).into())
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl FromValue for bool {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::Bool(value) => Ok(*value),
            _ => Err(invalid_value()),
        }
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Value::Bool(*self)
    }
}

macro_rules! integer_value {
    ($type:ty, $variant:ident) => {
        impl FromValue for $type {
            fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
                match value {
                    Value::Signed(value) => Self::try_from(*value).map_err(|_| invalid_value()),
                    Value::Unsigned(value) => Self::try_from(*value).map_err(|_| invalid_value()),
                    _ => Err(invalid_value()),
                }
            }
        }

        impl ToValue for $type {
            fn to_value(&self) -> Value {
                Value::$variant((*self).into())
            }
        }
    };
}

integer_value!(i32, Signed);
integer_value!(i64, Signed);
integer_value!(u32, Unsigned);
integer_value!(u64, Unsigned);

impl FromValue for u8 {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::Signed(value) => Self::try_from(*value).map_err(|_| invalid_value()),
            Value::Unsigned(value) => Self::try_from(*value).map_err(|_| invalid_value()),
            _ => Err(invalid_value()),
        }
    }
}

impl ToValue for u8 {
    fn to_value(&self) -> Value {
        Value::Unsigned(u64::from(*self))
    }

    fn byte_buffer(values: &[Self]) -> Option<Vec<u8>> {
        Some(values.to_vec())
    }
}

impl FromValue for usize {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::Signed(value) => Self::try_from(*value).map_err(|_| invalid_value()),
            Value::Unsigned(value) => Self::try_from(*value).map_err(|_| invalid_value()),
            _ => Err(invalid_value()),
        }
    }
}

impl ToValue for usize {
    fn to_value(&self) -> Value {
        Value::Unsigned(*self as u64)
    }
}

impl FromValue for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::Real(value) => Ok(*value),
            Value::Signed(value) => Ok(*value as Self),
            Value::Unsigned(value) => Ok(*value as Self),
            _ => Err(invalid_value()),
        }
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Value::Real(*self)
    }
}

impl FromValue for f32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        Ok(f64::from_value(value)? as Self)
    }
}

impl ToValue for f32 {
    fn to_value(&self) -> Value {
        Value::Real(f64::from(*self))
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::Null => Ok(None),
            value => T::from_value(value).map(Some),
        }
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self) -> Value {
        self.as_ref().map_or(Value::Null, T::to_value)
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::Array(values) => values.iter().map(T::from_value).collect(),
            Value::Data(bytes) => bytes
                .iter()
                .map(|&byte| T::from_value(&Value::Unsigned(u64::from(byte))))
                .collect(),
            _ => Err(invalid_value()),
        }
    }
}

impl<T: ToValue> ToValue for [T] {
    fn to_value(&self) -> Value {
        T::byte_buffer(self).map_or_else(
            || Value::Array(self.iter().map(T::to_value).collect()),
            Value::Data,
        )
    }
}

impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self) -> Value {
        T::byte_buffer(self).map_or_else(
            || Value::Array(self.iter().map(T::to_value).collect()),
            Value::Data,
        )
    }
}

impl<T: FromValue> FromValue for BTreeMap<String, T> {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        let Value::Dictionary(values) = value else {
            return Err(invalid_value());
        };
        values
            .iter()
            .map(|(key, value)| Ok((key.clone(), T::from_value(value)?)))
            .collect()
    }
}

impl<T: ToValue> ToValue for BTreeMap<String, T> {
    fn to_value(&self) -> Value {
        Value::Dictionary(
            self.iter()
                .map(|(key, value)| (key.clone(), value.to_value()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectHandle(pub *mut c_void);

impl FromValue for ObjectHandle {
    fn from_value(value: &Value) -> Result<Self, AVPlayerError> {
        match value {
            Value::Object(pointer) => Ok(Self(*pointer as *mut c_void)),
            _ => Err(invalid_value()),
        }
    }
}

impl ToValue for ObjectHandle {
    fn to_value(&self) -> Value {
        Value::Object(self.0 as usize)
    }
}

#[derive(Debug)]
pub struct InputValue {
    root: AVPValue,
    bytes: Vec<Box<[u8]>>,
    arrays: Vec<Box<[AVPValue]>>,
    keys: Vec<Box<[AVPBytes]>>,
}

impl InputValue {
    pub(crate) fn new<T: ToValue + ?Sized>(value: &T) -> Result<Self, AVPlayerError> {
        let mut input = Self {
            root: Self::empty(),
            bytes: Vec::new(),
            arrays: Vec::new(),
            keys: Vec::new(),
        };
        input.root = input.store(value.to_value(), 0)?;
        Ok(input)
    }

    pub(crate) const fn as_ptr(&self) -> *const AVPValue {
        &raw const self.root
    }

    const fn empty() -> AVPValue {
        AVPValue {
            kind: 0,
            reserved: 0,
            signed_value: 0,
            unsigned_value: 0,
            real_value: 0.0,
            bytes: AVPBytes {
                data: ptr::null(),
                length: 0,
            },
            values: ptr::null(),
            keys: ptr::null(),
            count: 0,
            object: ptr::null_mut(),
            owner: ptr::null_mut(),
        }
    }

    fn store_bytes(&mut self, value: Vec<u8>) -> AVPBytes {
        let value = value.into_boxed_slice();
        let bytes = AVPBytes {
            data: value.as_ptr(),
            length: value.len(),
        };
        self.bytes.push(value);
        bytes
    }

    fn store(&mut self, value: Value, depth: usize) -> Result<AVPValue, AVPlayerError> {
        if depth > 128 {
            return Err(AVPlayerError::InvalidArgument(
                "binary bridge value exceeds the depth limit".into(),
            ));
        }
        let mut raw = Self::empty();
        match value {
            Value::Null => {}
            Value::Bool(value) => {
                raw.kind = 1;
                raw.unsigned_value = u64::from(value);
            }
            Value::Signed(value) => {
                raw.kind = 2;
                raw.signed_value = value;
            }
            Value::Unsigned(value) => {
                raw.kind = 3;
                raw.unsigned_value = value;
            }
            Value::Real(value) => {
                raw.kind = 4;
                raw.real_value = value;
            }
            Value::String(value) => {
                raw.kind = 5;
                raw.bytes = self.store_bytes(value.into_bytes());
            }
            Value::Data(value) => {
                raw.kind = 6;
                raw.bytes = self.store_bytes(value);
            }
            Value::Array(values) => {
                raw.kind = 7;
                let values = values
                    .into_iter()
                    .map(|value| self.store(value, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice();
                raw.values = values.as_ptr();
                raw.count = values.len();
                self.arrays.push(values);
            }
            Value::Dictionary(values) => {
                raw.kind = 8;
                let (keys, values): (Vec<_>, Vec<_>) = values.into_iter().unzip();
                let keys = keys
                    .into_iter()
                    .map(|key| self.store_bytes(key.into_bytes()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let values = values
                    .into_iter()
                    .map(|value| self.store(value, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice();
                raw.keys = keys.as_ptr();
                raw.values = values.as_ptr();
                raw.count = values.len();
                self.keys.push(keys);
                self.arrays.push(values);
            }
            Value::Object(pointer) => {
                raw.kind = 9;
                raw.object = pointer as *mut c_void;
            }
        }
        Ok(raw)
    }
}

#[derive(Debug)]
pub struct InputString {
    input: InputValue,
}

impl InputString {
    pub(crate) fn new<T: Into<Vec<u8>>>(value: T) -> Result<Self, std::ffi::NulError> {
        let value = std::ffi::CString::new(value)?.into_bytes();
        let mut input = InputValue {
            root: InputValue::empty(),
            bytes: Vec::new(),
            arrays: Vec::new(),
            keys: Vec::new(),
        };
        input.root.kind = 5;
        input.root.bytes = input.store_bytes(value);
        Ok(Self { input })
    }

    pub(crate) const fn as_ptr(&self) -> *const AVPValue {
        &raw const self.input.root
    }
}

unsafe fn read_slice<'a, T>(pointer: *const T, count: usize) -> Result<&'a [T], AVPlayerError> {
    if count == 0 {
        return Ok(&[]);
    }
    if pointer.is_null()
        || count > isize::MAX as usize / core::mem::size_of::<T>().max(1)
        || pointer.align_offset(core::mem::align_of::<T>()) != 0
    {
        return Err(invalid_value());
    }
    Ok(unsafe { core::slice::from_raw_parts(pointer, count) })
}

unsafe fn read_bytes(bytes: AVPBytes) -> Result<Vec<u8>, AVPlayerError> {
    Ok(unsafe { read_slice(bytes.data, bytes.length)? }.to_vec())
}

unsafe fn read_node(pointer: *const AVPValue, depth: usize) -> Result<Value, AVPlayerError> {
    if pointer.is_null() || depth > 128 {
        return Err(invalid_value());
    }
    let value = unsafe { &*pointer };
    Ok(match value.kind {
        0 => Value::Null,
        1 if value.unsigned_value <= 1 => Value::Bool(value.unsigned_value != 0),
        2 => Value::Signed(value.signed_value),
        3 => Value::Unsigned(value.unsigned_value),
        4 => Value::Real(value.real_value),
        5 => Value::String(
            String::from_utf8(unsafe { read_bytes(value.bytes)? }).map_err(|_| invalid_value())?,
        ),
        6 => Value::Data(unsafe { read_bytes(value.bytes)? }),
        7 => Value::Array(
            unsafe { read_slice(value.values, value.count)? }
                .iter()
                .map(|value| unsafe { read_node(value, depth + 1) })
                .collect::<Result<_, _>>()?,
        ),
        8 => {
            let keys = unsafe { read_slice(value.keys, value.count)? };
            let values = unsafe { read_slice(value.values, value.count)? };
            Value::Dictionary(
                keys.iter()
                    .zip(values)
                    .map(|(key, value)| {
                        let key = String::from_utf8(unsafe { read_bytes(*key)? })
                            .map_err(|_| invalid_value())?;
                        Ok((key, unsafe { read_node(value, depth + 1)? }))
                    })
                    .collect::<Result<_, AVPlayerError>>()?,
            )
        }
        9 => Value::Object(value.object as usize),
        _ => return Err(invalid_value()),
    })
}

pub unsafe fn read_value(pointer: *const AVPValue) -> Result<Value, AVPlayerError> {
    unsafe { read_node(pointer, 0) }
}

pub unsafe fn take_value(pointer: *mut AVPValue) -> Result<Value, AVPlayerError> {
    struct Owner(*mut AVPValue);

    impl Drop for Owner {
        fn drop(&mut self) {
            unsafe { ffi::avp_value_free(self.0) };
        }
    }

    let owner = Owner(pointer);
    unsafe { read_node(owner.0, 0) }
}

pub unsafe fn read_string(pointer: *const AVPValue) -> Result<String, AVPlayerError> {
    let value = unsafe { read_node(pointer, 0)? };
    String::from_value(&value)
}

pub unsafe fn take_string(pointer: *mut AVPValue) -> Result<String, AVPlayerError> {
    let value = unsafe { take_value(pointer)? };
    String::from_value(&value)
}
