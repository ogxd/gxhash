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
    compress_all::<true>(input, seed)
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
// more (see prepare_seed), so that sparse seed differences don't meet the equally sparse differences of raw
// blocks. It is xored with a constant first, as AES rounds preserve symmetric states (eg all bytes equal from
// a zero seed). Inputs of less than 16 bytes are a single block, which can meet the raw seed.
#[inline(always)]
pub(crate) unsafe fn compress_all<const GXHASH: bool>(input: &[u8], seed: State) -> State {

    let len = input.len();
    let ptr = input.as_ptr() as *const State;

    if len < VECTOR_SIZE {
        let hash = if len == 0 {
            lane_end(lane_start(seed, create_empty()))
        } else {
            // Input fits on a single SIMD vector, however we might read beyond the input message
            // Thus we need this safe method that checks if it can safely read beyond or must copy.
            // Padding bytes are set to the input length, which makes the vector unique for each input.
            lane_end(lane_start(seed, get_partial(ptr, len)))
        };
        // Finalized separately from larger inputs, which have a pending xor
        return finish::<GXHASH>(hash);
    }

    // The hash is (hash ^ pending): see lane_end_xor
    let (hash, pending) = if len <= VECTOR_SIZE * 8 {
        let seed = prepare_seed::<GXHASH>(seed);
        if len == VECTOR_SIZE {
            // A single block too, but without padding. Using the prepared seed makes it independent of the
            // above, as the difference of a full block and of a padded block could otherwise match the length
            // difference after a round.
            lane_end_xor(lane_start(seed, load_unaligned(ptr)), load_len(len))
        } else {
            compress_upto_128(ptr, len, seed)
        }
    } else {
        // Finalizing in the callee keeps the caller from saving anything across the call
        return compress_large::<GXHASH>(ptr, len, prepare_seed::<GXHASH>(seed));
    };

    if GXHASH { finalize_xor(hash, pending) } else { xor(hash, pending) }
}

#[inline(always)]
unsafe fn prepare_seed<const GXHASH: bool>(seed: State) -> State {
    if GXHASH { lane_end(lane_start(seed, ld(KEYS.as_ptr()))) } else { seed }
}

// Inputs of 17 to 128 bytes. Each size class extends the work of the smaller one (early exit rather than
// jumping into the steps), so that steps are not duplicated and small inputs don't jump over steps.
#[inline(always)]
unsafe fn compress_upto_128(ptr: *const State, len: usize, seed: State) -> (State, State) {
    let end = ptr.cast::<u8>().add(len).cast::<State>();
    let mut a = lane_start(seed, load_unaligned(ptr));
    a = absorb_after_two_rounds(a, load_unaligned(end.sub(1)));
    if len > VECTOR_SIZE * 2 {
        a = absorb_after_two_rounds(a, load_unaligned(ptr.add(1)));
        if len > VECTOR_SIZE * 3 {
            a = absorb_after_two_rounds(a, load_unaligned(end.sub(2)));
            if len > VECTOR_SIZE * 4 {
                let mut b = lane_start(seed, load_unaligned(ptr.add(2)));
                b = absorb_after_two_rounds(b, load_unaligned(end.sub(3)));
                if len > VECTOR_SIZE * 6 {
                    b = absorb_after_two_rounds(b, load_unaligned(ptr.add(3)));
                    b = absorb_after_two_rounds(b, load_unaligned(end.sub(4)));
                }
                // merge(a, b) ^ len == merge(a, b ^ len)
                let (b, pending) = lane_end_xor(b, load_len(len));
                return (merge(lane_end(a), b), pending);
            }
        }
    }
    lane_end_xor(a, load_len(len))
}

// Absorbs a block into a lane after two rounds, so that the previous block went through two rounds when it
// meets this one
#[inline(always)]
unsafe fn absorb_after_two_rounds(lane: State, block: State) -> State {
    lane_absorb(lane_absorb(lane, create_empty()), block)
}

// gxhash finalizes the hash, while the Hasher only does it in finish
#[inline(always)]
pub(crate) unsafe fn finish<const GXHASH: bool>(hash: State) -> State {
    if GXHASH { finalize(hash) } else { hash }
}

// Inputs of more than 128 bytes. Not inlined, which keeps the inlined bytecode small, as these do enough work
// for the call to be negligible. Uses the C ABI so that vectors are passed in registers (the Rust ABI passes
// them through the stack on x86). This function is internal, so its vector types being FFI-safe doesn't matter.
#[allow(improper_ctypes_definitions)]
#[inline(never)]
unsafe extern "C" fn compress_large<const GXHASH: bool>(ptr: *const State, len: usize, seed: State) -> State {

    let hash = if len <= VECTOR_SIZE * 32 {
        compress_lanes::<4>(ptr, len, seed)
    } else if len <= VECTOR_SIZE * 128 {
        compress_lanes::<8>(ptr, len, seed)
    } else {
        // Includes the length and finalizes as well, so that this is a tail call and this function needs no
        // stack frame
        return compress_16::<GXHASH>(ptr, len, seed);
    };

    finish::<GXHASH>(xor(hash, load_len(len)))
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
    merge_lanes(lanes)
}

// 16 lanes for large inputs. 8 lanes are enough to saturate 128-bit AES units, but CPUs with 256-bit wide AES
// (hybrid) process two lanes per instruction and need 16 lanes. Fewer lanes are used for smaller inputs, as
// merging lanes costs rounds too. Expects len > 32 * VECTOR_SIZE.
// Not inlined: 16 lanes use many registers, which would otherwise make smaller inputs pay for saving them.
#[cfg(not(all(feature = "hybrid", any(target_arch = "x86", target_arch = "x86_64"))))]
#[allow(improper_ctypes_definitions)]
#[inline(never)]
unsafe extern "C" fn compress_16<const GXHASH: bool>(ptr: *const State, len: usize, seed: State) -> State {
    finish::<GXHASH>(xor(compress_lanes::<16>(ptr, len, seed), load_len(len)))
}

// Merges lanes absorbing a block per round, folding the lanes array in half at each level (lane i with lane
// i + L/2). Folding in half (rather than merging neighbours) allows wider implementations to merge several
// lanes per instruction. The first level merges with a single round, F(a) ^ b: the last blocks of a then went
// through two rounds when meeting the last blocks of b, which went through one. These can only cancel out for
// structured differences of four bits or more, with a 2^-32 probability. Next levels use merge.
#[inline(always)]
pub(crate) unsafe fn merge_lanes<const L: usize>(mut lanes: [State; L]) -> State {
    let mut n = L;
    while n > 1 {
        n /= 2;
        for i in 0..n {
            lanes[i] = if n == L / 2 { aes_encrypt(lanes[i], lanes[i + n]) } else { merge(lanes[i], lanes[i + n]) };
        }
    }
    lanes[0]
}

// F(F(a)) ^ b, so that the blocks of a went through three rounds when they meet the blocks of b
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
            (0, [0x9e2a74d6323c1e7e, 0xd638e4bb0b02c811, 0xed134192d1162902, 0x5e3515741951d182]),
            (1, [0xbd1d4a1906c5d74a, 0xfd039c40ccc371b6, 0x7ee15d54f5363997, 0xf4945e7dab41f445]),
            (7, [0x7b52fc7f0b725c24, 0x3af624c643b91323, 0x3d5bde93db94bed4, 0x935d237644019699]),
            (15, [0x102cac080b0aaa2f, 0x62054297e134d44f, 0x8e68b70dda47714d, 0x9ca5ba6ced8c590a]),
            (16, [0xe15d40b9c011ade5, 0xffc7d0191ea25a21, 0x27016e699ed0947a, 0x426f7b5b3cc5e8fe]),
            (17, [0x40791975698e21b9, 0x088af766ea213939, 0x7b8cc3a63d500b3b, 0xe66543dcdbc639ba]),
            (31, [0xf6b8d10583aff73d, 0x002835620b7a9da3, 0xd4d13870ab672dcd, 0x672940b6f9a96639]),
            (32, [0xb5537031f3aac5e6, 0xad5b853ba5ec1b91, 0xa0e900022dd1dd61, 0x0892398b93f46fee]),
            (33, [0x9b0f0e8ff1f2a1e3, 0x8081e03938e07534, 0x2f3a86e4ef700ec4, 0xefeef9375d900a3b]),
            (48, [0xca85ec03c3b4c895, 0xcf187c3b94aa71d2, 0xce07b046f69bce61, 0x868720b88891d1f0]),
            (49, [0xaf2aafa09a85e34d, 0xa8d482de6c246dac, 0xdd4b388229918521, 0x4ae234a2a904edcd]),
            (64, [0x4ead24efb9175e7f, 0x7df7bd82c441b97a, 0xca293a74699ce2ad, 0x9cf06b773e171912]),
            (65, [0xc15aa42eb759647e, 0x810e86cf1c97cde2, 0x115651a649cad9fa, 0x61e911fed6c80240]),
            (96, [0x5445b21eb455d334, 0xb992281d6a3a524a, 0xa63a91ca55a2daa9, 0x607d9cf50ce6eb71]),
            (97, [0x9a158f9c19c009d8, 0x4bea47008ddff769, 0xffe65eb56b966166, 0x4fc8ab14afe1cf9a]),
            (128, [0x1171957e32035f02, 0x77b7ecce0299a2bb, 0xcde87a297a20af8c, 0x946067321102656f]),
            (129, [0xbe4a24e6191efca2, 0xb16365ebf4c609f5, 0x6eb45e35d5dd0e81, 0xcb666635bf4d821e]),
            (256, [0x34b670299cd81782, 0x00370d479a205b09, 0xc59ac15f702127d9, 0x01dfbe8d63e79eab]),
            (257, [0x30f5d0e8c1a96b51, 0xd1f653fa179697df, 0x451164ed40e36944, 0x8448d8ca4069fff7]),
            (1024, [0x104643c594a0e8af, 0x3211829ad48bbeb0, 0x2be12306d9469dab, 0xa128dea3318d0954]),
            (1025, [0xa6f201c56698e466, 0x7cd067502cd74948, 0x25d0e13ba0f559d5, 0x99084eceaf21451c]),
            (4242, [0x5417103f34a4621a, 0xf87cf5c9f35c92b2, 0xc4d3ceee561116d6, 0xecb1a829277b7663]),
        ];
        for (len, hashes) in expected {
            for (seed, hash) in seeds.iter().zip(hashes) {
                assert_eq!(hash, gxhash64(&data[..len], *seed), "len {len}, seed {seed}");
            }
        }
        assert_eq!(3930652002, gxhash32(b"Hello World", 0));
        assert_eq!(0xc270913193acccf4aa07094dea48fd62, gxhash128(b"Hello World", 0));
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
