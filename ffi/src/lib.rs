use core::slice;

#[no_mangle]
pub unsafe extern "C" fn gxhash0_32(buf: *const (), len: usize, seed: i32) -> u32 {
    let data: &[u8] = slice::from_raw_parts(buf as *const u8, len);
    gxhash::gxhash0_32(data, seed)
}

#[no_mangle]
pub unsafe extern "C" fn gxhash0_64(buf: *const (), len: usize, seed: i32) -> u64 {
    let data: &[u8] = slice::from_raw_parts(buf as *const u8, len);
    gxhash::gxhash0_64(data, seed)
}

#[no_mangle]
pub unsafe extern "C" fn gxhash1_32(buf: *const (), len: usize, seed: i32) -> u32 {
    let data: &[u8] = slice::from_raw_parts(buf as *const u8, len);
    gxhash::gxhash1_32(data, seed)
}

#[no_mangle]
pub unsafe extern "C" fn gxhash1_64(buf: *const (), len: usize, seed: i32) -> u64 {
    let data: &[u8] = slice::from_raw_parts(buf as *const u8, len);
    gxhash::gxhash1_64(data, seed)
}