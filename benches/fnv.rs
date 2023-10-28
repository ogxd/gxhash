const INITIAL_STATE: u64 = 0xcbf29ce484222325;
const PRIME: u64 = 0x100000001b3;

#[inline]
pub const fn fnv_hash(bytes: &[u8]) -> u64 {
    let mut hash = INITIAL_STATE;
    let mut i = 0;
    while i < bytes.len() {
        hash = hash ^ (bytes[i] as u64);
        hash = hash.wrapping_mul(PRIME);
        i += 1;
    }
    hash
}