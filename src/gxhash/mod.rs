pub(crate) mod platform;

use platform::*;

/// Hashes an arbitrary stream of bytes to an u32.
///
/// # Example
///
/// ```
/// let bytes = [42u8; 1000];
/// let seed = 1234;
/// println!("Hash is {:x}!", gxhash::gxhash32(&bytes, seed));
/// ```
#[inline(always)]
pub fn gxhash32(input: &[u8], seed: i64) -> u32 {
    unsafe {
        let p = &gxhash(input, create_seed(seed)) as *const State as *const u32;
        *p
    }
}

/// Hashes an arbitrary stream of bytes to an u64.
///
/// # Example
///
/// ```
/// let bytes = [42u8; 1000];
/// let seed = 1234;
/// println!("Hash is {:x}!", gxhash::gxhash64(&bytes, seed));
/// ```
#[inline(always)]
pub fn gxhash64(input: &[u8], seed: i64) -> u64 {
    unsafe {
        let p = &gxhash(input, create_seed(seed)) as *const State as *const u64;
        *p
    }
}

/// Hashes an arbitrary stream of bytes to an u128.
///
/// # Example
///
/// ```
/// let bytes = [42u8; 1000];
/// let seed = 1234;
/// println!("Hash is {:x}!", gxhash::gxhash128(&bytes, seed));
/// ```
#[inline(always)]
pub fn gxhash128(input: &[u8], seed: i64) -> u128 {
    unsafe {
        let p = &gxhash(input, create_seed(seed)) as *const State as *const u128;
        *p
    }
}

#[inline(always)]
pub(crate) unsafe fn gxhash(input: &[u8], seed: State) -> State {
    finalize(compress_all::<true>(input, seed))
}

// Blocks of 16 bytes are absorbed with AES rounds (F: SubBytes, ShiftRows and MixColumns, without key).
// When the differences of two blocks meet in a xor, they may cancel each other out with a probability that
// depends on how many rounds each went through: after one round, a sparse difference is still sparse, and
// after two rounds, it spans a 32-bit space only, which the sparse differences of other blocks can match.
// So whenever two block differences meet, either one went through three rounds, or one went through two
// and the other none (the latter then being as sparse as the input difference):
// - Up to 128 bytes, lanes absorb blocks with two rounds between blocks, and are merged as F(F(a)) ^ b.
// - Beyond, lanes absorb one block per round (lane = F(lane ^ block)) for throughput. Sparse differences of
//   6 bits or more on consecutive blocks of a lane can then cancel each other out, with a probability that
//   depends on the seed.
// Inputs of more than 16 bytes are read as chunks from both ends, overlapping when needed, which avoids
// masking and keeps lanes independent (high ILP). Inputs of different lengths may then have the same blocks,
// so the length is xored to the result, which must go through a round before being combined with anything
// else. A length difference spans one or two bytes, and can only be matched by the difference of a block
// that went through a single round with a 2^-32 probability. Inputs of less than 16 bytes are padded with
// their length instead, and inputs of 16 bytes use another seed than theirs.
// Seeds are differences too: a seed given by users goes through a round before meeting blocks of 16 bytes or
// more (PREPARE_SEED), so that sparse seed differences don't meet the equally sparse differences of raw
// blocks. It is xored with a constant first, as AES rounds preserve symmetric states (eg all bytes equal from
// a zero seed). Inputs of less than 16 bytes are a single block, which can meet the raw seed.
#[inline(always)]
pub(crate) unsafe fn compress_all<const PREPARE_SEED: bool>(input: &[u8], seed: State) -> State {

    let len = input.len();
    let ptr = input.as_ptr() as *const State;

    if len < VECTOR_SIZE {
        if len == 0 {
            return lane_end(lane_start(seed, create_empty()));
        }
        // Input fits on a single SIMD vector, however we might read beyond the input message
        // Thus we need this safe method that checks if it can safely read beyond or must copy.
        // Padding bytes are set to the input length, which makes the vector unique for each input.
        return lane_end(lane_start(seed, get_partial(ptr, len)));
    }

    let seed = if PREPARE_SEED { lane_end(lane_start(seed, ld(KEYS.as_ptr()))) } else { seed };

    if len == VECTOR_SIZE {
        // A single block too, but without padding. Using the prepared seed makes it independent of the
        // above, as the difference of a full block and of a padded block could otherwise match the length
        // difference after a round.
        return xor(lane_end(lane_start(seed, load_unaligned(ptr))), load_len(len));
    }

    if len <= VECTOR_SIZE * 2 {
        return xor(compress_chains::<1, 2>(ptr, len, seed), load_len(len));
    }
    if len <= VECTOR_SIZE * 3 {
        return xor(compress_chains::<1, 3>(ptr, len, seed), load_len(len));
    }
    if len <= VECTOR_SIZE * 4 {
        return xor(compress_chains::<1, 4>(ptr, len, seed), load_len(len));
    }

    compress_large(ptr, len, seed)
}

// Inputs of more than 64 bytes. Not inlined, which keeps the inlined bytecode small, as these do enough work
// for the call to be negligible. Uses the C ABI so that vectors are passed in registers (the Rust ABI passes
// them through the stack on x86). This function is internal, so its vector types being FFI-safe doesn't matter.
#[allow(improper_ctypes_definitions)]
#[inline(never)]
unsafe extern "C" fn compress_large(ptr: *const State, len: usize, seed: State) -> State {

    let hash = if len <= VECTOR_SIZE * 6 {
        compress_chains::<2, 3>(ptr, len, seed)
    } else if len <= VECTOR_SIZE * 8 {
        compress_chains::<2, 4>(ptr, len, seed)
    } else if len <= VECTOR_SIZE * 32 {
        compress_lanes::<4>(ptr, len, seed)
    } else if len <= VECTOR_SIZE * 128 {
        compress_lanes::<8>(ptr, len, seed)
    } else {
        // Already includes the length
        return compress_16(ptr, len, seed);
    };

    xor(hash, load_len(len))
}

// L lanes, each absorbing C blocks with two rounds between blocks: F(F(..F(F(F(seed ^ b0)) ^ b1)..) ^ bn).
// Chunks of L blocks are read from the start then from the end of the input.
// Expects L * (C - 1) * VECTOR_SIZE < len <= L * C * VECTOR_SIZE.
#[inline(always)]
unsafe fn compress_chains<const L: usize, const C: usize>(ptr: *const State, len: usize, seed: State) -> State {
    let end = ptr.cast::<u8>().add(len).cast::<State>();
    let chunk = |c: usize| if c < (C + 1) / 2 { ptr.add(c * L) } else { end.sub((C - c) * L) };
    let mut lanes = [seed; L];
    for i in 0..L {
        let mut lane = lane_start(seed, load_unaligned(chunk(0).add(i)));
        for c in 1..C {
            lane = lane_absorb(lane_absorb(lane, create_empty()), load_unaligned(chunk(c).add(i)));
        }
        lanes[i] = lane_end(lane);
    }
    merge_lanes::<L, false>(lanes)
}

// L lanes, each absorbing a block per round, reading chunks of L blocks. The last chunk is aligned on the
// end of the input, and may overlap the previous one. Expects len > 2 * L * VECTOR_SIZE.
#[inline(always)]
unsafe fn compress_lanes<const L: usize>(ptr: *const State, len: usize, seed: State) -> State {
    let last = ptr.cast::<u8>().add(len - L * VECTOR_SIZE).cast::<State>();
    let mut lanes = [seed; L];
    for i in 0..L {
        lanes[i] = lane_start(seed, load_unaligned(ptr.add(i)));
    }
    let mut ptr = ptr.add(L);
    while ptr < last {
        for i in 0..L {
            lanes[i] = lane_absorb(lanes[i], load_unaligned(ptr.add(i)));
        }
        ptr = ptr.add(L);
    }
    for i in 0..L {
        lanes[i] = lane_end(lane_absorb(lanes[i], load_unaligned(last.add(i))));
    }
    merge_lanes::<L, true>(lanes)
}

// 16 lanes for large inputs. 8 lanes are enough to saturate 128-bit AES units, but CPUs with 256-bit wide AES
// (hybrid) process two lanes per instruction and need 16 lanes. Fewer lanes are used for smaller inputs, as
// merging lanes costs rounds too. Expects len > 32 * VECTOR_SIZE.
// Not inlined: 16 lanes use many registers, which would otherwise make smaller inputs pay for saving them.
#[cfg(not(all(feature = "hybrid", any(target_arch = "x86", target_arch = "x86_64"))))]
#[allow(improper_ctypes_definitions)]
#[inline(never)]
unsafe extern "C" fn compress_16(ptr: *const State, len: usize, seed: State) -> State {
    xor(compress_lanes::<16>(ptr, len, seed), load_len(len))
}

// Merges lanes pairwise, folding the lanes array in half at each level (lane i with lane i + L/2).
// Folding in half (rather than merging neighbours) allows wider implementations to merge several
// lanes per instruction. Lanes are merged as F(F(a)) ^ b, so that the blocks of a went through three
// rounds when they meet the blocks of b.
// Lanes absorbing a block per round (FAST) are merged with a single round at the first level: the last
// blocks of a then went through two rounds when meeting the last blocks of b, which went through one. These
// can only cancel out for structured differences of four bits or more, with a 2^-32 probability.
#[inline(always)]
pub(crate) unsafe fn merge_lanes<const L: usize, const FAST: bool>(mut lanes: [State; L]) -> State {
    let mut n = L;
    while n > 1 {
        n /= 2;
        for i in 0..n {
            lanes[i] = if FAST && n == L / 2 { aes_encrypt(lanes[i], lanes[i + n]) } else { merge(lanes[i], lanes[i + n]) };
        }
    }
    lanes[0]
}

#[inline(always)]
pub(crate) unsafe fn merge(a: State, b: State) -> State {
    aes_encrypt(aes_encrypt(a, create_empty()), b)
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::Rng;

    #[test]
    fn all_blocks_are_consumed() {
        for s in 1..1200 {
            let mut bytes = vec![42u8; s];
            let ref_hash = gxhash32(&bytes, 0);

            for i in 0..bytes.len() {
                let swap = bytes[i];
                bytes[i] = 82;
                let new_hash = gxhash32(&bytes, 0);
                bytes[i] = swap;

                assert_ne!(ref_hash, new_hash, "byte {i} not processed for input of size {s}");
            }
        }
    }

    #[test]
    fn add_zeroes_mutates_hash() {
        let mut bytes = [0u8; 1200];

        let mut rng = rand::thread_rng();
        rng.fill(&mut bytes[..32]);

        let mut ref_hash = 0;

        for i in 32..100 {
            let new_hash = gxhash32(&mut bytes[..i], 0);
            assert_ne!(ref_hash, new_hash, "Same hash at size {i} ({new_hash})");
            ref_hash = new_hash;
        }
    }

    #[test]
    fn does_not_hash_outside_of_bounds() {
        let mut bytes = [0u8; 1200];
        const OFFSET: usize = 100;

        let mut rng = rand::thread_rng();
        rng.fill(bytes.as_mut_slice());

        for i in 1..1000 {
            let hash = gxhash32(&bytes[OFFSET..i+OFFSET], 42);
            // We change the bytes right before and after the input slice. It shouldn't alter the hash.
            rng.fill(&mut bytes[..OFFSET]);
            rng.fill(&mut bytes[i+OFFSET..]);
            let new_hash = gxhash32(&bytes[OFFSET..i+OFFSET], 42);
            assert_eq!(new_hash, hash, "Hashed changed for input size {i} ({new_hash} != {hash})");
        }
    }

    #[test]
    fn hash_of_zero_is_not_zero() {
        assert_ne!(0, gxhash32(&[0u8; 0], 0));
        assert_ne!(0, gxhash32(&[0u8; 1], 0));
        assert_ne!(0, gxhash32(&[0u8; 1200], 0));
    }

    #[test]
    fn is_stable() {
        // Hashes must be the same on all platforms, with or without the hybrid feature
        let data: Vec<u8> = (0..4242usize).map(|i| (i * 31 + 7) as u8).collect();
        let seeds = [0i64, 42, -1, i64::MIN];
        let expected: [(usize, [u64; 4]); 22] = [
            (0, [0xeed96c4396ffe83f, 0x53d06b308c2af58a, 0x3b1c1da2950a844f, 0x63e926ac7983fc77]),
            (1, [0xe320e6240e32080d, 0xfb299163a2015951, 0xcf90fa4eff6fb382, 0x3e22a8b94aef0cb0]),
            (7, [0x0ba48cf18cd389ac, 0x5a3bf1379cf5bf81, 0xac86222bf445bce2, 0x9596035fd84481ff]),
            (15, [0xba4c134b8c2407a8, 0xaaa2ce0fe53ab4db, 0xb1f49b38caf459ae, 0xa81db75bd280cc65]),
            (16, [0x7996cf7427122bdf, 0x9b0e4e24a0206b45, 0xda3fc5af5e7df32e, 0x8ec4b536f032a0af]),
            (17, [0xcbf82b341116d208, 0x0d1b062fc3a3b3a1, 0x0b8cf425e7d6c936, 0x2ae4474df425b364]),
            (31, [0x6b3a76e86cdb1ebb, 0xb9c940798c871702, 0xcab187ce4a7e66ab, 0x6952cfe583f77aa1]),
            (32, [0x5687d5a8707a8b56, 0xc786c1c81a992c31, 0x73dc32dcfe05ba00, 0x0d183c817fb81ad3]),
            (33, [0xd5387aec618ab5c2, 0x767b9bb207a9dc04, 0x1d83f5aaaca3b7a3, 0x75b6026842820226]),
            (48, [0xa9852a5d8353e303, 0x8bd7b594da025d09, 0x94ff416289c344b3, 0x9deeadc54b612459]),
            (49, [0x8d36e4858496b3f2, 0xba0cba8eb0da8e66, 0xe6bf65608d03a90f, 0xefe71d78dcd5257c]),
            (64, [0x46dce96ba62de214, 0xc84354a1ce1f92bf, 0xfe280840c9784370, 0x877cc5b98c5a0d97]),
            (65, [0x9b724404e21674d4, 0x8e6c4ac6deb1ee57, 0xfee5d881bf596a99, 0x32f8b70ca850d687]),
            (96, [0x79ee988db0950277, 0x24b13632d9ed53c3, 0x6b810088a49c2b7b, 0x4a028ca6274ae782]),
            (97, [0xe5a1e0d009fa561d, 0xb4922907d0c33592, 0x1c6a73003caea1bb, 0xf6f770795e39846c]),
            (128, [0x54188e42dd8126eb, 0x2ed500bd10b01469, 0xf3862e33b9f67247, 0xb87afee5fa8c14b4]),
            (129, [0xa319f1e279c50fc9, 0x94e0fb213c25b199, 0x329ea851b4c3b7a9, 0xd6275d516df35b52]),
            (256, [0x999fd573ef28fd77, 0xb9af3e207b979820, 0x9aab57bdcea353ec, 0x0f2d6f68c23f8fc7]),
            (257, [0x43de4e90b9f7df33, 0x683b4f0e7e214ca2, 0x1711789563705a24, 0x415b8a406396c3fd]),
            (1024, [0xbaa047a3cdf6a098, 0x521196ec6778bca5, 0x589003cf569817c7, 0xe1c922fa2002b16d]),
            (1025, [0xdc6d28a357ede39e, 0x1f12dd15f15150a7, 0x1012c0c82346cc79, 0x450c298843a37448]),
            (4242, [0xd3bfcbd35211b9b2, 0xf05449da70b59092, 0x55692f0a1d1210f1, 0xa5f3467322abd384]),
        ];
        for (len, hashes) in expected {
            for (seed, hash) in seeds.iter().zip(hashes) {
                assert_eq!(hash, gxhash64(&data[..len], *seed), "len {len}, seed {seed}");
            }
        }
        assert_eq!(3285473415, gxhash32(b"Hello World", 0));
        assert_eq!(0x68613d434bf54f2f876cf5c6c3d45887, gxhash128(b"Hello World", 0));
    }

    #[test]
    fn inputs_of_different_lengths_do_not_collide() {
        for seed in [0, 42, -1] {
            // A 16-byte input ending with 0xFF vs the 15-byte input with all bytes incremented
            let mut a = [0u8; 16];
            a[15] = 0xFF;
            assert_ne!(gxhash64(&a, seed), gxhash64(&[1u8; 15], seed));
            // A 15-byte input vs the same bytes followed by its padding
            let b: Vec<u8> = (0..15).collect();
            let mut c = b.clone();
            c.push(15);
            assert_ne!(gxhash64(&b, seed), gxhash64(&c, seed));
            // A 31-byte input vs the 32-byte input made of its padded first 15 bytes and its last 16 bytes
            let d: Vec<u8> = (0..31u8).map(|i| i.wrapping_mul(37).wrapping_add(11)).collect();
            let mut e: Vec<u8> = d[..15].iter().map(|x| x.wrapping_add(15)).collect();
            e.push(15);
            e.extend_from_slice(&d[15..]);
            assert_ne!(gxhash64(&d, seed), gxhash64(&e, seed));
        }
    }

    // Keys with a few bits set, of the same length or not. A difference of a few bits that went through a
    // single AES round is still sparse, and can be cancelled by the sparse difference of another block.
    #[test]
    fn sparse_inputs_do_not_collide() {
        fn flip_bits(key: &mut [u8], start: usize, bits_left: usize, hashes: &mut Vec<u64>) {
            hashes.push(gxhash64(key, 0));
            if bits_left == 0 {
                return;
            }
            for i in start..key.len() * 8 {
                key[i / 8] ^= 1 << (i % 8);
                flip_bits(key, i + 1, bits_left - 1, hashes);
                key[i / 8] ^= 1 << (i % 8);
            }
        }
        fn assert_no_collisions(mut hashes: Vec<u64>) {
            let count = hashes.len();
            hashes.sort_unstable();
            hashes.dedup();
            assert_eq!(count, hashes.len(), "some sparse inputs collide");
        }
        // Up to 2 bits set, all lengths in the same set
        let mut hashes = Vec::new();
        for len in 0..=40 {
            flip_bits(&mut vec![0u8; len], 0, 2, &mut hashes);
        }
        assert_no_collisions(hashes);
        let mut hashes = Vec::new();
        flip_bits(&mut vec![0u8; 64], 0, 2, &mut hashes);
        assert_no_collisions(hashes);
        let mut hashes = Vec::new();
        flip_bits(&mut vec![0u8; 20], 0, 3, &mut hashes);
        assert_no_collisions(hashes);
    }
}
