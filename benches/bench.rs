extern crate dsfmt;
extern crate test;

use test::Bencher;

use std::rand::{Rng, SeedableRng};
use dsfmt::DSFMTRng;

#[bench]
fn bench_gen_1_000_000_rands(b: &mut Bencher){
    let mut rng: DSFMTRng = SeedableRng::from_seed(1);

    b.iter(||{
        for _ in range(1u, 1_000_000){
            rng.next_f64();
        }
    });
}
