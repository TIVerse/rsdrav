#![no_main]

use libfuzzer_sys::fuzz_target;
use rsdrav::prelude::*;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    
    let width = u16::from_le_bytes([data[0], data[1]]).clamp(10, 1000);
    let height = u16::from_le_bytes([data[2], data[3]]).clamp(10, 500);
    
    let rect = Rect::new(0, 0, width, height);
    
    // Fuzz Row layout
    let row = Row::new(rect)
        .add(Length::Fixed(10))
        .add(Length::Fill(1))
        .add(Length::Percent(0.3));
    let _ = row.calculate();
    
    // Fuzz Column layout
    let col = Column::new(rect)
        .add(Length::Fixed(5))
        .add(Length::Fill(2));
    let _ = col.calculate();
});
