#![no_main]

use libfuzzer_sys::fuzz_target;
use rsdrav::prelude::*;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    let sig = Signal::new(0i32);
    
    for &byte in data {
        match byte % 3 {
            0 => sig.set(byte as i32),
            1 => sig.update(|v| *v = (*v).wrapping_add(byte as i32)),
            _ => { let _ = sig.get(); }
        }
    }
});
