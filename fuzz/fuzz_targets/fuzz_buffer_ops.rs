#![no_main]

use libfuzzer_sys::fuzz_target;
use rsdrav::prelude::*;

fuzz_target!(|data: &[u8]| {
    if data.len() < 6 {
        return;
    }
    
    let width = u16::from_le_bytes([data[0], data[1]]).clamp(1, 200);
    let height = u16::from_le_bytes([data[2], data[3]]).clamp(1, 100);
    let x = u16::from_le_bytes([data[4], data[5]]) % width;
    let y = if data.len() > 7 {
        u16::from_le_bytes([data[6], data[7]]) % height
    } else {
        0
    };
    
    let mut buffer = Buffer::new(width, height);
    let cell = Cell::new('X');
    
    // Fuzz buffer operations
    buffer.set(x, y, cell);
    let _ = buffer.get(x, y);
    buffer.clear();
});
