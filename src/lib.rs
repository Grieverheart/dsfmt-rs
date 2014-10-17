#![crate_name = "dsfmt"]
#![crate_type = "lib"]

#![feature(simd)]
#![feature(asm)]

pub use mt19937::DSFMTRng;

mod mt19937;

fn main(){
    let mut rng = DSFMTRng::from_seed(1);
    let mut sum = 0.0f64;
    for _ in range(0i, 1_000_000_000){
        sum += rng.genrand_close_open();
    }
    println!("{}", sum);
}
