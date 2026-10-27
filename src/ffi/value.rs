use core::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AVPBytes {
    pub data: *const u8,
    pub length: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AVPValue {
    pub kind: u32,
    pub reserved: u32,
    pub signed_value: i64,
    pub unsigned_value: u64,
    pub real_value: f64,
    pub bytes: AVPBytes,
    pub values: *const Self,
    pub keys: *const AVPBytes,
    pub count: usize,
    pub object: *mut c_void,
    pub owner: *mut c_void,
}

const _: () = {
    assert!(core::mem::size_of::<AVPBytes>() == 16);
    assert!(core::mem::size_of::<AVPValue>() == 88);
    assert!(core::mem::offset_of!(AVPValue, bytes) == 32);
    assert!(core::mem::offset_of!(AVPValue, owner) == 80);
};
