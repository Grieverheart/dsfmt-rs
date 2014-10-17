#![crate_name = "dsfmt"]
#![crate_type = "lib"]

#![feature(simd)]
#![feature(asm)]

pub use mt19937::DSFMTRng;

mod mt19937;
