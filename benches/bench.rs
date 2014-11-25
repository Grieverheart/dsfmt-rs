extern crate dsfmt;
extern crate test;

use test::Bencher;

use std::rand::{Rng, SeedableRng, Isaac64Rng, XorShiftRng};
use dsfmt::DSFMTRng;

#[bench]
fn bench_mt19937_1_000_000_rands(b: &mut Bencher){
    let mut rng: DSFMTRng = SeedableRng::from_seed(1);

    b.iter(||{
        for _ in range(1u, 1_000_000){
            rng.next_f64();
        }
    });
}

#[bench]
fn bench_isaac64_1_000_000_rands(b: &mut Bencher){
    let mut rng: Isaac64Rng = SeedableRng::from_seed([1u64].as_slice());

    b.iter(||{
        for _ in range(1u, 1_000_000){
            rng.next_f64();
        }
    });
}

#[bench]
fn bench_xor_1_000_000_rands(b: &mut Bencher){
    let mut rng: XorShiftRng = SeedableRng::from_seed([1u32, 1u32, 1u32, 1u32]);

    b.iter(||{
        for _ in range(1u, 1_000_000){
            rng.next_f64();
        }
    });
}
