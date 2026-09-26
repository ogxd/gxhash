// Backends: the hardware one relies on AES instructions, and the portable one computes the same hashes without them
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "platform/x86.rs"]
pub(crate) mod hw;
#[cfg(target_arch = "aarch64")]
#[path = "platform/arm.rs"]
pub(crate) mod hw;
#[path = "platform/soft.rs"]
mod soft;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub(crate) use hw::State;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) type State = [u8; 16];

pub(crate) const VECTOR_SIZE: usize = core::mem::size_of::<State>();

pub(crate) const KEYS: [u32; 12] =
   [0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E,
    0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39132BD9,
    0xD0012E32, 0x689D2B7D, 0x5544B1B7, 0xC78B122B];

// Hints that the branch is unlikely, so that the likely branch is the one that doesn't jump
#[rustversion::since(1.95)]
#[inline(always)]
fn cold_path() {
    core::hint::cold_path()
}

#[rustversion::before(1.95)]
#[inline(always)]
fn cold_path() {}

// Runs an operation on the hardware backend, inlined, when its features are enabled at compile time. Otherwise, calls
// it on the backend chosen at runtime (see outlined in algorithm.rs).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
macro_rules! dispatch {
    ($op:ident($($arg:expr),*)) => {
        unsafe {
            if hw::STATIC {
                hw::$op($($arg),*)
            } else if hw::has_aes() {
                hw::outlined::$op($($arg),*)
            } else {
                cold_path();
                soft::outlined::$op($($arg),*)
            }
        }
    };
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
macro_rules! dispatch {
    ($op:ident($($arg:expr),*)) => { unsafe { soft::outlined::$op($($arg),*) } };
}

// The operations, on the backend chosen by dispatch
pub(crate) mod dispatched {
    use super::*;

    #[inline(always)]
    pub(crate) fn hash(input: &[u8], seed: State) -> State {
        dispatch!(hash(input, seed))
    }

    #[inline(always)]
    pub(crate) fn absorb(state: State, bytes: &[u8]) -> State {
        dispatch!(absorb(state, bytes))
    }

    #[inline(always)]
    pub(crate) fn absorb_u64(state: State, value: u64) -> State {
        dispatch!(absorb_u64(state, value))
    }

    #[inline(always)]
    pub(crate) fn absorb_u128(state: State, value: u128) -> State {
        dispatch!(absorb_u128(state, value))
    }

    #[inline(always)]
    pub(crate) fn finalize(state: State) -> State {
        dispatch!(finalize(state))
    }
}

// Several operations, which with_backend runs with a single dispatch
pub(crate) trait Run {
    type Output;

    // Runs the operations on the backend chosen by dispatch, one by one
    fn run(self) -> Self::Output;

    // Runs the operations on the hardware backend, once detected at runtime. Must be #[inline(always)], so that it is
    // compiled in with_features, with the features of the backend: its operations are then inlined in it. The default
    // is for platforms without hardware backend, where it is never called.
    unsafe fn run_hw(self) -> Self::Output where Self: Sized {
        self.run()
    }
}

// Runs r in a single call compiled with the features of the hardware backend, when it is detected at runtime
#[inline(always)]
pub(crate) fn with_backend<R: Run>(r: R) -> R::Output {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    if !hw::STATIC && hw::has_aes() {
        return unsafe { hw::outlined::with_features(r) };
    }
    r.run()
}

// States from and to u128, with the same byte order on all platforms
#[inline(always)]
pub(crate) fn from_u128(x: u128) -> State {
    unsafe { core::mem::transmute(x.to_le_bytes()) }
}

#[inline(always)]
pub(crate) fn to_u128(state: State) -> u128 {
    u128::from_le_bytes(unsafe { core::mem::transmute(state) })
}

#[inline(always)]
pub(crate) fn create_seed(seed: i64) -> State {
    let seed = seed as u64 as u128;
    from_u128(seed << 64 | seed)
}

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
    to_u128(dispatched::hash(input, create_seed(seed))) as u32
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
    to_u128(dispatched::hash(input, create_seed(seed))) as u64
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
    to_u128(dispatched::hash(input, create_seed(seed)))
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
        // Hashes must be the same on all platforms and backends
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

    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    #[test]
    fn backends_compute_the_same_hashes() {
        if !hw::has_aes() {
            return;
        }
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; 4416];
        rng.fill(data.as_mut_slice());
        for len in 0..=4400 {
            let input = &data[len % 16..len % 16 + len];
            let (state, value) = (from_u128(rng.gen()), rng.gen::<u128>());
            unsafe {
                assert_eq!(to_u128(hw::outlined::hash(input, state)), to_u128(soft::outlined::hash(input, state)), "length {len}");
                assert_eq!(to_u128(hw::outlined::absorb(state, input)), to_u128(soft::outlined::absorb(state, input)), "length {len}");
                assert_eq!(to_u128(hw::outlined::absorb_u64(state, value as u64)), to_u128(soft::outlined::absorb_u64(state, value as u64)));
                assert_eq!(to_u128(hw::outlined::absorb_u128(state, value)), to_u128(soft::outlined::absorb_u128(state, value)));
                assert_eq!(to_u128(hw::outlined::finalize(state)), to_u128(soft::outlined::finalize(state)));
            }
        }
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
