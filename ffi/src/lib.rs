use core::slice;

#[no_mangle]
pub unsafe extern "C" fn gxhash32(buf: *const (), len: usize, seed: u64) -> u32 {
    let data: &[u8] = slice::from_raw_parts(buf as *const u8, len);
    gxhash::gxhash32(data, seed)
}

#[no_mangle]
pub unsafe extern "C" fn gxhash64(buf: *const (), len: usize, seed: u64) -> u64 {
    let data: &[u8] = slice::from_raw_parts(buf as *const u8, len);
    gxhash::gxhash64(data, seed)
}