#![no_main]

use libfuzzer_sys::fuzz_target;
use rsdrav::prelude::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Fuzz the command parser with arbitrary input
        let _ = Command::parse(s);
    }
});
