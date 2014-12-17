#![crate_name = "dsfmt"]
#![crate_type = "lib"]

#![feature(simd)]

pub use mt19937::DSFMTRng;

mod mt19937;
