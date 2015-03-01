#![crate_name = "dsfmt"]
#![crate_type = "lib"]

#![feature(core)]
#![feature(simd)]

extern crate rand;

pub use mt19937::DSFMTRng;

mod mt19937;
