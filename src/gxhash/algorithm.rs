// The algorithm, included in each backend (see platform/*.rs), where it uses the primitives of that backend. The
// backends compile the functions of the algorithm that are not inlined with the features they require, as these
// may only be detected at runtime. See with_features.

use super::*;

// 4KiB is the default page size for most systems, and conservative for other systems such as macOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

#[inline(always)]
unsafe fn get_partial(p: *const State, len: usize) -> State {
    // Safety check. Miri can't run the inline assembly of the read beyond the input, so it always takes the other path.
    if !cfg!(miri) && check_same_page(p) {
        get_partial_unsafe(p, len)
    } else {
        cold_path();
        get_partial_safe(p, len)
    }
}

#[inline(always)]
unsafe fn check_same_page(ptr: *const State) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits
    let offset_within_page = address & (PAGE_SIZE - 1);
    // Check if the 16th byte from the current offset exceeds the page boundary
    offset_within_page < PAGE_SIZE - VECTOR_SIZE
}

#[inline(always)]
pub(crate) unsafe fn finalize(hash: State) -> State {
    finalize_xor(hash, create_empty())
}

// Finalizes hash ^ pending
#[inline(always)]
unsafe fn finalize_xor(hash: State, pending: State) -> State {
    // A single key is enough to break the symmetry of states preserved by AES rounds. Other keys would only
    // xor constants to intermediate states, which doesn't change the quality, but costs a register each.
    let mut hash = aes_encrypt_xor(hash, pending, ld(KEYS.as_ptr()));
    hash = aes_encrypt(hash, create_empty());
    hash = aes_encrypt_last(hash, create_empty());

    hash
}

#[inline(always)]
pub(crate) unsafe fn hash(input: &[u8], seed: State) -> State {
    compress_all::<true>(input, seed)
}

// Hasher::write. The state seeds the compression, after a round so that the previous write went through two
// rounds when it meets these bytes. Finalization only happens in finish.
#[inline(always)]
pub(crate) unsafe fn absorb(state: State, bytes: &[u8]) -> State {
    let hash = compress_all::<false>(bytes, aes_encrypt(state, ld(KEYS.as_ptr())));
    if bytes.len() < VECTOR_SIZE {
        hash
    } else {
        // See compress_all: the length it includes must go through a round before the next write
        aes_encrypt(hash, create_empty())
    }
}

// Hasher::write_* for integers. The value of the previous write went through two rounds when it meets this one.
// The key breaks the symmetry of states such as the one of a zero seed.
#[inline(always)]
pub(crate) unsafe fn absorb_u64(state: State, value: u64) -> State {
    aes_encrypt(aes_encrypt(state, load_u64(value)), ld(KEYS.as_ptr()))
}

#[inline(always)]
pub(crate) unsafe fn absorb_u128(state: State, value: u128) -> State {
    aes_encrypt(aes_encrypt(state, load_u128(value)), ld(KEYS.as_ptr()))
}

// The operations above, compiled with the features of the backend so that the runtime dispatch can call them (see
// gxhash::dispatch). Callers compiled with these features may still inline them.
pub(crate) mod outlined {
    use super::*;

    with_features! { inline:
        #[allow(improper_ctypes_definitions)]
        pub(crate) unsafe extern "C" fn hash(input: &[u8], seed: State) -> State {
            super::hash(input, seed)
        }

        #[allow(improper_ctypes_definitions)]
        pub(crate) unsafe extern "C" fn absorb(state: State, bytes: &[u8]) -> State {
            super::absorb(state, bytes)
        }

        #[allow(improper_ctypes_definitions)]
        pub(crate) unsafe extern "C" fn absorb_u64(state: State, value: u64) -> State {
            super::absorb_u64(state, value)
        }

        #[allow(improper_ctypes_definitions)]
        pub(crate) unsafe extern "C" fn absorb_u128(state: State, value: u128) -> State {
            super::absorb_u128(state, value)
        }

        #[allow(improper_ctypes_definitions)]
        pub(crate) unsafe extern "C" fn finalize(state: State) -> State {
            super::finalize(state)
        }

        // See Run
        #[allow(dead_code)]
        pub(crate) unsafe fn with_features<R: Run>(r: R) -> R::Output {
            r.run_hw()
        }
    }
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
unsafe fn compress_all<const GXHASH: bool>(input: &[u8], seed: State) -> State {

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

with_features! {
    // Inputs of more than 128 bytes. Not inlined, which keeps the inlined bytecode small, as these do enough work
    // for the call to be negligible. Uses the C ABI so that vectors are passed in registers (the Rust ABI passes
    // them through the stack on x86). This function is internal, so its vector types being FFI-safe doesn't matter.
    #[allow(improper_ctypes_definitions)]
    unsafe extern "C" fn compress_large<const GXHASH: bool>(ptr: *const State, len: usize, seed: State) -> State {

        let hash = if len <= VECTOR_SIZE * 32 {
            compress_lanes::<4>(ptr, len, seed)
        } else if len <= VECTOR_SIZE * 128 {
            compress_lanes::<8>(ptr, len, seed)
        } else if has_wide_aes() {
            // Same as compress_16, processing two lanes per instruction
            return compress_16_wide::<GXHASH>(ptr, len, seed);
        } else {
            // Includes the length and finalizes as well, so that this is a tail call and this function needs no
            // stack frame
            return compress_16::<GXHASH>(ptr, len, seed);
        };

        finish::<GXHASH>(xor(hash, load_len(len)))
    }

    // 16 lanes for large inputs. 8 lanes are enough to saturate 128-bit AES units, but CPUs with 256-bit wide AES
    // (see compress_16_wide) process two lanes per instruction and need 16 lanes. Fewer lanes are used for smaller
    // inputs, as merging lanes costs rounds too. Expects len > 32 * VECTOR_SIZE.
    // Not inlined: 16 lanes use many registers, which would otherwise make smaller inputs pay for saving them.
    #[allow(improper_ctypes_definitions)]
    pub(crate) unsafe extern "C" fn compress_16<const GXHASH: bool>(ptr: *const State, len: usize, seed: State) -> State {
        finish::<GXHASH>(xor(compress_lanes::<16>(ptr, len, seed), load_len(len)))
    }
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

// Merges lanes absorbing a block per round, folding the lanes array in half at each level (lane i with lane
// i + L/2). Folding in half (rather than merging neighbours) allows wider implementations to merge several
// lanes per instruction. The first level merges with a single round, F(a) ^ b: the last blocks of a then went
// through two rounds when meeting the last blocks of b, which went through one. These can only cancel out for
// structured differences of four bits or more, with a 2^-32 probability. Next levels use merge.
#[inline(always)]
unsafe fn merge_lanes<const L: usize>(mut lanes: [State; L]) -> State {
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
