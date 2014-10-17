use std::mem;
use std::iter::range_inclusive;

const DSFMT_MEXP:uint = 19937;
const DSFMT_N:   uint = ((DSFMT_MEXP - 128) / 104 + 1);
const DSFMT_N64: uint = (DSFMT_N * 2);
const DSFMT_SR:  uint = 12;

const DSFMT_LOW_MASK:   u64 = 0x000FFFFFFFFFFFFF;
const DSFMT_HIGH_CONST: u64 = 0x3FF0000000000000;

const DSFMT_POS1: uint = 117;
const DSFMT_SL1:  uint = 19;
const DSFMT_PCV1: u64  = 0x3d84e1ac0dc82880;
const DSFMT_PCV2: u64  = 0x0000000000000001;
const DSFMT_FIX1: u64  = 0x90014964b32f4329;
const DSFMT_FIX2: u64  = 0x3b8d12ac548a7c7a;
const DSFMT_MSK1: u64  = 0x000ffafffffffb3f;
const DSFMT_MSK2: u64  = 0x000ffdfffc90fffd;

const SSE2_SHUFF: u8 = 0x1b;
const SSE2_PARAMS_MASK: u64x2 = u64x2{x: DSFMT_MSK1, y: DSFMT_MSK2};

pub struct DSFMTRng{
    status: [u64x2, ..DSFMT_N + 1],
    idx: uint,
}

#[repr(C)]
#[simd]
#[deriving(Show)]
struct u32x4{
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

#[repr(C)]
#[simd]
#[deriving(Show)]
struct u64x2{
    x: u64,
    y: u64,
}

impl Index<uint, u64> for u64x2{
    fn index(&self, _rhs: &uint) -> &u64{
        match *_rhs{
            0 => &self.x,
            _ => &self.y,
        }
    }
}

impl IndexMut<uint, u64> for u64x2{
    fn index_mut(&mut self, _rhs: &uint) -> &mut u64{
        match *_rhs{
            0 => &mut self.x,
            _ => &mut self.y,
        }
    }
}

impl Index<uint, u32> for u32x4{
    fn index(&self, _rhs: &uint) -> &u32{
        match *_rhs{
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => &self.w,
        }
    }
}

impl IndexMut<uint, u32> for u32x4{
    fn index_mut(&mut self, _rhs: &uint) -> &mut u32{
        match *_rhs{
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => &mut self.w,
        }
    }
}

/**
 * This function simulate a 32-bit array index overlapped to 64-bit
 * array of LITTLE ENDIAN in BIG ENDIAN machine.
 */
#[cfg(target_endian = "big")]
fn idxof(i: uint) -> uint {
    i ^ 1
}

#[cfg(target_endian = "little")]
fn idxof(i: uint) -> uint {
    i
}

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

#[allow(unused)]
fn do_recursion(r: &mut u64x2, a: u64x2, b: u64x2, u: &mut u64x2){
    let mut z = a;
    unsafe{asm!(
        "psllq  $4, $0\n\t
         pxor   $8, $0\n\t
         pshufd $3, $1, $1\n\t
         pxor   $0, $1\n\t
         movaps $1, $0\n\t
         movaps $1, $2\n\t
         pand   $6, $0\n\t
         psrlq  $5, $2\n\t
         pxor   $7, $2\n\t
         pxor   $0, $2\n\t
        "
        :"+x"(z), "+x"(*u), "=x"(*r)
        :"i"(SSE2_SHUFF), "i"(DSFMT_SL1), "i"(DSFMT_SR), "x"(SSE2_PARAMS_MASK), "m"(a), "x"(b)
    )};
}

impl DSFMTRng{
    pub fn new() -> DSFMTRng {
        DSFMTRng{status: [u64x2{x: 0, y: 0}, ..DSFMT_N + 1], idx: 0}
    }

    pub fn from_seed(seed: u32) -> DSFMTRng {
        let mut rng = DSFMTRng::new();
        rng.chk_init_gen_rand(seed);
        rng
    }

    /**
     * This function initializes the internal state array to fit the IEEE
     * 754 format.
     */
    fn initial_mask(&mut self){
        for i in range(0, DSFMT_N * 2){
            let (n, m) = (i / 2, i % 2);
            self.status[n][m] = (self.status[n][m] & DSFMT_LOW_MASK) | DSFMT_HIGH_CONST;
        }
    }

    /**
     * This function certificate the period of 2^{SFMT_MEXP}-1.
     * @param dsfmt dsfmt state vector.
     */
    fn period_certification(&mut self) {
        let pcv: [u64, ..2] = [DSFMT_PCV1, DSFMT_PCV2];

        {
            let tmp = [self.status[DSFMT_N][0] ^ DSFMT_FIX1, self.status[DSFMT_N][1] ^ DSFMT_FIX2];
            let mut inner = (tmp[0] & pcv[0]) ^ (tmp[1] & pcv[1]);

            let mut i = 32u;
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
            for i in range_inclusive(1, 0){
                let mut work = 1u64;
                for _ in range(0i, 64){
                    if (work & pcv[i]) != 0 {
                        self.status[DSFMT_N][i] ^= work;
                        return;
                    }
                    work <<= 1;
                }
            }
        }
    }

    fn chk_init_gen_rand(&mut self, seed: u32) {
        let psfmt: &mut [u32x4, ..DSFMT_N + 1] = unsafe{mem::transmute(&mut self.status)};
        let i = idxof(0);
        psfmt[i / 4][i % 4] = seed;

        for i in range(1u, 4 * (DSFMT_N + 1)){
            let idx1 = idxof(i);
            let idx2 = idxof(i - 1);
            psfmt[idx1 / 4][idx1 % 4] = 1812433253u32 * (psfmt[idx2 / 4][idx2 % 4] ^ (psfmt[idx2 / 4][idx2 % 4] >> 30)) + (i as u32);
        }

        self.initial_mask();
        self.period_certification();
        self.idx = DSFMT_N64;
    }

    fn gen_rand_all(&mut self){
        let mut lung = self.status[DSFMT_N];
        {
            let a = self.status[0];
            let b = self.status[DSFMT_POS1];
            do_recursion(&mut self.status[0], a, b, &mut lung);
        }
        let mut i = 1u;
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

    pub fn genrand_close1_open2(&mut self) -> f64 {
        if self.idx >= DSFMT_N64 {
            self.gen_rand_all();
            self.idx = 0;
        }

        let (n, m) = (self.idx / 2, self.idx % 2);
        self.idx += 1;

        unsafe{mem::transmute(self.status[n][m])}
    }

    pub fn genrand_close_open(&mut self) -> f64 {
        self.genrand_close1_open2() - 1.0
    }
}

