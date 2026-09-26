// Portable backend, for CPUs without the AES instructions of the hardware backend. It computes the same hashes,
// much slower. AES rounds use lookup tables, whose access times depend on the state.

use super::{cold_path, Run, State, KEYS, VECTOR_SIZE};

// Nothing to enable. Never inlined, so that the dispatch doesn't inline this backend next to the hardware one.
macro_rules! with_features {
    (inline: $($item:item)*) => { $(#[inline(never)] $item)* };
    ($($item:item)*) => { $(#[inline(never)] $item)* };
}

#[path = "../algorithm.rs"]
mod algorithm;
pub(crate) use algorithm::*;

#[inline(always)]
pub(crate) fn has_wide_aes() -> bool {
    false
}
pub(crate) use algorithm::compress_16 as compress_16_wide;

// The bytes of a state, in the order of the vector lanes of the hardware backends (little endian)
#[inline(always)]
fn to_bytes(state: State) -> [u8; 16] {
    unsafe { core::mem::transmute(state) }
}

#[inline(always)]
fn from_bytes(bytes: [u8; 16]) -> State {
    unsafe { core::mem::transmute(bytes) }
}

const fn mul2(x: u8) -> u8 {
    (x << 1) ^ if x & 0x80 != 0 { 0x1B } else { 0 }
}

// AES S-box: affine transformation of the multiplicative inverse in GF(2^8)
const SBOX: [u8; 256] = {
    let mut sbox = [0x63; 256];
    // p runs through all non-zero elements as powers of 3, and q through their inverses as powers of 1/3
    let (mut p, mut q) = (1u8, 1u8);
    loop {
        p ^= mul2(p);
        q ^= q << 1;
        q ^= q << 2;
        q ^= q << 4;
        if q & 0x80 != 0 {
            q ^= 0x09;
        }
        sbox[p as usize] = 0x63 ^ q ^ q.rotate_left(1) ^ q.rotate_left(2) ^ q.rotate_left(3) ^ q.rotate_left(4);
        if p == 1 {
            break sbox;
        }
    }
};

// SubBytes then MixColumns of a byte in the first row, as a column of 4 bytes (little endian). Rotating it by
// 8 * r bits gives the column of a byte in row r.
const MIX: [u32; 256] = {
    let mut mix = [0; 256];
    let mut i = 0;
    while i < 256 {
        let s = SBOX[i];
        mix[i] = u32::from_le_bytes([mul2(s), s, s, mul2(s) ^ s]);
        i += 1;
    }
    mix
};

// ShiftRows moves the byte of row r in column c to column c - r, so byte i of the result comes from byte 5 * i
#[inline(always)]
fn shifted(bytes: &[u8; 16], i: usize) -> usize {
    bytes[(5 * i) % 16] as usize
}

// AES round without key: SubBytes, ShiftRows and MixColumns
#[inline(always)]
fn round(state: State) -> State {
    let bytes = to_bytes(state);
    let mut out = [0; 16];
    for c in 0..4 {
        let mut column = 0;
        for r in 0..4 {
            column ^= MIX[shifted(&bytes, 4 * c + r)].rotate_left(8 * r as u32);
        }
        out[4 * c..4 * c + 4].copy_from_slice(&column.to_le_bytes());
    }
    from_bytes(out)
}

// Last AES round without key: SubBytes and ShiftRows
#[inline(always)]
fn last_round(state: State) -> State {
    let bytes = to_bytes(state);
    let mut out = [0; 16];
    for i in 0..16 {
        out[i] = SBOX[shifted(&bytes, i)];
    }
    from_bytes(out)
}

#[inline(always)]
pub unsafe fn create_empty() -> State {
    from_bytes([0; 16])
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const State) -> State {
    from_bytes(core::ptr::read_unaligned(p as *const [u8; 16]))
}

// Input bytes, followed by padding bytes set to the input length
#[inline(always)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    let mut bytes = [len as u8; 16];
    core::ptr::copy_nonoverlapping(data as *const u8, bytes.as_mut_ptr(), len);
    from_bytes(bytes)
}

// Reading beyond the input is left to the hardware backends
#[inline(always)]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    get_partial_safe(data, len)
}

// See the x86 backend for the semantics of these functions
#[inline(always)]
pub unsafe fn aes_encrypt(data: State, keys: State) -> State {
    xor(round(data), keys)
}

#[inline(always)]
pub unsafe fn aes_encrypt_last(data: State, keys: State) -> State {
    xor(last_round(data), keys)
}

#[inline(always)]
pub unsafe fn ld(array: *const u32) -> State {
    let mut bytes = [0; 16];
    for i in 0..4 {
        bytes[4 * i..4 * i + 4].copy_from_slice(&(*array.add(i)).to_le_bytes());
    }
    from_bytes(bytes)
}

#[inline(always)]
pub unsafe fn lane_start(key: State, block: State) -> State {
    xor(key, block)
}

#[inline(always)]
pub unsafe fn lane_absorb(lane: State, block: State) -> State {
    aes_encrypt(lane, block)
}

#[inline(always)]
pub unsafe fn lane_end(lane: State) -> State {
    aes_encrypt(lane, create_empty())
}

#[inline(always)]
pub unsafe fn lane_end_xor(lane: State, x: State) -> (State, State) {
    (aes_encrypt(lane, x), create_empty())
}

#[inline(always)]
pub unsafe fn aes_encrypt_xor(data: State, pending: State, keys: State) -> State {
    aes_encrypt(xor(data, pending), keys)
}

#[inline(always)]
pub unsafe fn xor(a: State, b: State) -> State {
    let x = u128::from_ne_bytes(to_bytes(a)) ^ u128::from_ne_bytes(to_bytes(b));
    from_bytes(x.to_ne_bytes())
}

#[inline(always)]
pub unsafe fn load_len(len: usize) -> State {
    load_u64(len as u64)
}

#[inline(always)]
pub unsafe fn load_u64(x: u64) -> State {
    load_u128(x as u128)
}

#[inline(always)]
pub unsafe fn load_u128(x: u128) -> State {
    from_bytes(x.to_le_bytes())
}
