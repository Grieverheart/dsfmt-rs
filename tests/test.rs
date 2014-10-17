extern crate dsfmt;

use dsfmt::DSFMTRng;

#[test]
fn test_u32_values(){
    let mut rng = DSFMTRng::from_seed(1);
    for _ in range(1u, 1_000_000){
        rng.genrand_u32();
    }
    assert_eq!(rng.genrand_u32(), 2164198192);
}

#[test]
fn test_f64_close_open_values(){
    let mut rng = DSFMTRng::from_seed(1);
    for _ in range(1u, 1_000_000){
        rng.genrand_close_open();
    }
    assert_eq!(rng.genrand_close_open(), 0.38634062184491925f64);
}

#[test]
fn test_f64_open_open_values(){
    let mut rng = DSFMTRng::from_seed(1);
    for _ in range(1u, 1_000_000){
        rng.genrand_open_open();
    }
    assert_eq!(rng.genrand_open_open(), 0.3863406218449194375f64);
}
