use std::mem;
use rand::{Rng, SeedableRng, Rand};
use std::ops::{Index, IndexMut};

const DSFMT_LOW_MASK:   u64 = 0x000FFFFFFFFFFFFF;
const DSFMT_HIGH_CONST: u64 = 0x3FF0000000000000;

const DSFMT_F32_MANTISSA_MASK: u64 = 0x00000000007FFFFF;
const DSFMT_HIGH_CONST32:      u32 = 0x3F800000;

const DSFMT_MEXP:usize = 19937;
const DSFMT_N:   usize = ((DSFMT_MEXP - 128) / 104 + 1);
const DSFMT_N64: usize = (DSFMT_N * 2);
const DSFMT_SR:  usize = 12;

const DSFMT_POS1: usize = 117;
const DSFMT_SL1:  usize = 19;
const DSFMT_PCV1: u64  = 0x3d84e1ac0dc82880;
const DSFMT_PCV2: u64  = 0x0000000000000001;
const DSFMT_FIX1: u64  = 0x90014964b32f4329;
const DSFMT_FIX2: u64  = 0x3b8d12ac548a7c7a;
const DSFMT_MSK1: u64  = 0x000ffafffffffb3f;
const DSFMT_MSK2: u64  = 0x000ffdfffc90fffd;

const SSE2_PARAMS_MASK: u64x2 = u64x2{x: DSFMT_MSK1, y: DSFMT_MSK2};
const SSE2_SL: u64x2 = u64x2{x: DSFMT_SL1 as u64, y: DSFMT_SL1 as u64};
const SSE2_SR: u64x2 = u64x2{x: DSFMT_SR as u64, y: DSFMT_SR as u64};

pub struct DSFMTRng{
    status: [u64x2; DSFMT_N + 1],
    idx: usize,
}

#[repr(C)]
#[simd]
#[derive(Debug, Copy)]
struct u32x4{
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

#[repr(C)]
#[simd]
#[derive(Debug, Copy)]
struct u64x2{
    x: u64,
    y: u64,
}

impl Index<usize> for u64x2{
    type Output = u64;
    #[inline(always)]
    fn index(&self, _rhs: &usize) -> &u64{
        match *_rhs{
            0 => &self.x,
            _ => &self.y,
        }
    }
}

impl IndexMut<usize> for u64x2{
    #[inline(always)]
    fn index_mut(&mut self, _rhs: &usize) -> &mut u64{
        match *_rhs{
            0 => &mut self.x,
            _ => &mut self.y,
        }
    }
}

impl Index<usize> for u32x4{
    type Output = u32;
    #[inline(always)]
    fn index(&self, _rhs: &usize) -> &u32{
        match *_rhs{
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => &self.w,
        }
    }
}

impl IndexMut<usize> for u32x4{
    #[inline(always)]
    fn index_mut(&mut self, _rhs: &usize) -> &mut u32{
        match *_rhs{
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => &mut self.w,
        }
    }
}

#[cfg(target_endian = "big")]
fn idxof(i: usize) -> usize {
    i ^ 1
}

#[cfg(target_endian = "little")]
fn idxof(i: usize) -> usize {
    i
}

//#[allow(unused)]
//#[inline(always)]
//fn do_recursion(r: &mut u64x2, a: u64x2, b: u64x2, lung: &mut u64x2){
//
//    let u64x2{x: t0, y: t1} = a;
//    let u64x2{x: l0, y: l1} = *lung;
//
//    lung[0] = (t0 << DSFMT_SL1) ^ (l1 >> 32) ^ (l1 << 32) ^ b[0];
//    lung[1] = (t1 << DSFMT_SL1) ^ (l0 >> 32) ^ (l0 << 32) ^ b[1];
//
//    r[0] = ((*lung)[0] >> DSFMT_SR) ^ ((*lung)[0] & DSFMT_MSK1) ^ t0;
//    r[1] = ((*lung)[1] >> DSFMT_SR) ^ ((*lung)[1] & DSFMT_MSK2) ^ t1;
//}

#[inline(always)]
pub fn reverse_u32s(u: u64x2) -> u64x2 {
    unsafe {
        let tmp = mem::transmute::<_, u32x4>(u);
        let swapped = u32x4 {x: tmp.w, y: tmp.z, z: tmp.y, w: tmp.x};
        mem::transmute::<_, u64x2>(swapped)
    }
}

#[allow(unused)]
#[inline(always)]
fn do_recursion(r: &mut u64x2, a: u64x2, b: u64x2, u: &mut u64x2){
    let swapped = reverse_u32s(*u);
    let uu = (a << SSE2_SL) ^ b ^ swapped;

    *r = (uu >> SSE2_SR) ^ (uu & SSE2_PARAMS_MASK) ^ a;
    *u = uu;
}

impl DSFMTRng{
    /// Create a new unseeded DSFMTRng instance
    pub fn new_unseeded() -> DSFMTRng {
        DSFMTRng{status: [u64x2{x: 0, y: 0}; DSFMT_N + 1], idx: 0}
    }

    /// Initializes the internal state array to fit the IEEE 754 format.
    fn initial_mask(&mut self){
        for i in 0..DSFMT_N * 2 {
            let (n, m) = (i / 2, i % 2);
            self.status[n][m] = (self.status[n][m] & DSFMT_LOW_MASK) | DSFMT_HIGH_CONST;
        }
    }

    /// Certifies the period of 2^{SFMT_MEXP}-1.
    fn period_certification(&mut self) {
        let pcv: [u64; 2] = [DSFMT_PCV1, DSFMT_PCV2];

        {
            let tmp = [self.status[DSFMT_N][0] ^ DSFMT_FIX1, self.status[DSFMT_N][1] ^ DSFMT_FIX2];
            let mut inner = (tmp[0] & pcv[0]) ^ (tmp[1] & pcv[1]);

            let mut i: usize = 32;
            while i > 0 {
                inner ^= inner >> i;
                i >>= 1;
            }
            inner &= 1;
            /* check OK */
            if inner == 1 {
                return;
            }
        }

        /* check NG, and modification */
        if (DSFMT_PCV2 & 1) == 1 {
            self.status[DSFMT_N][1] ^= 1;
        }
        else{
            for i in (0..2).rev() {
                let mut work = 1u64;
                for _ in 0..64{
                    if (work & pcv[i]) != 0 {
                        self.status[DSFMT_N][i] ^= work;
                        return;
                    }
                    work <<= 1;
                }
            }
        }
    }

    fn init(&mut self, seed: u32) {
        let psfmt: &mut [u32x4; DSFMT_N + 1] = unsafe{mem::transmute(&mut self.status)};
        let i = idxof(0);
        psfmt[i / 4][i % 4] = seed;

        for i in 1..4 * (DSFMT_N + 1) {
            let idx1 = idxof(i);
            let idx2 = idxof(i - 1);
            psfmt[idx1 / 4][idx1 % 4] = 1812433253u32 * (psfmt[idx2 / 4][idx2 % 4] ^ (psfmt[idx2 / 4][idx2 % 4] >> 30)) + (i as u32);
        }

        self.initial_mask();
        self.period_certification();
        self.idx = DSFMT_N64;
    }

    #[inline(never)]
    fn gen_rand_all(&mut self){
        let mut lung = self.status[DSFMT_N];
        {
            let a = self.status[0];
            let b = self.status[DSFMT_POS1];
            do_recursion(&mut self.status[0], a, b, &mut lung);
        }
        let mut i: usize = 1;
        while i < DSFMT_N - DSFMT_POS1 {
            let a = self.status[i];
            let b = self.status[i + DSFMT_POS1];
            do_recursion(&mut self.status[i], a, b, &mut lung);
            i += 1;
        }
        while i < DSFMT_N {
            let a = self.status[i];
            let b = self.status[i + DSFMT_POS1 - DSFMT_N];
            do_recursion(&mut self.status[i], a, b, &mut lung);
            i += 1;
        }
        self.status[DSFMT_N] = lung;
    }

    #[inline(always)]
    fn next(&mut self) -> u64 {
        if self.idx >= DSFMT_N64 {
            self.gen_rand_all();
            self.idx = 0;
        }

        let n = self.idx;
        self.idx += 1;

        let v = unsafe {
            mem::transmute::<_, &[u64; 2 * (DSFMT_N + 1)]>(&self.status)
        };
        v[n]
    }

    #[inline]
    pub fn genrand_close1_open2(&mut self) -> f64 {
        unsafe{mem::transmute(self.next())}
    }

    #[inline]
    pub fn genrand_open_open(&mut self) -> f64 {
        unsafe{mem::transmute::<u64, f64>(self.next() | 1u64) - 1.0f64}
    }
}

impl Rng for DSFMTRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        (self.next() & 0xffffffffu64) as u32
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        unsafe{mem::transmute::<u32, f32>(((self.next() & DSFMT_F32_MANTISSA_MASK)) as u32 | DSFMT_HIGH_CONST32) - 1.0}
    }

    #[inline]
    fn next_f64(&mut self) -> f64 {
        self.genrand_close1_open2() - 1.0
    }
}

impl SeedableRng<u32> for DSFMTRng {
    fn reseed(&mut self, seed: u32){
        self.init(seed);
    }

    fn from_seed(seed: u32) -> DSFMTRng {
        let mut rng = DSFMTRng::new_unseeded();
        rng.init(seed);
        rng
    }
}

impl Rand for DSFMTRng {
    fn rand<R: Rng>(other: &mut R) -> DSFMTRng {
        SeedableRng::from_seed(other.next_u32())
    }
}
