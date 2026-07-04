use std::fs;

use bencher::{benchmark_group, benchmark_main, black_box, Bencher};

#[cfg(unix)]
const TEXT_PATH: &str = "benches/data/wikipedia-rust.txt";

#[cfg(windows)]
const TEXT_PATH: &str = r"benches\data\wikipedia-rust.txt";

static UTF8_CHAR_WIDTH: [usize; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

const BALANCED_FIRST_BYTE_COUNT: usize = 16 * 1024;

fn mix_index(mut value: u32) -> u32 {
    value = value.wrapping_add(0x9E37_79B9);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^ (value >> 16)
}

fn first_byte_for_class(class: usize, mixed: u32) -> u8 {
    match class {
        0 => (mixed & 0x7F) as u8,
        1 => 0xC2 + (mixed % 30) as u8,
        2 => 0xE0 + (mixed % 16) as u8,
        _ => 0xF0 + (mixed % 5) as u8,
    }
}

fn make_balanced_first_bytes() -> Vec<u8> {
    let mut first_bytes = Vec::with_capacity(BALANCED_FIRST_BYTE_COUNT);

    for i in 0..BALANCED_FIRST_BYTE_COUNT {
        first_bytes.push(first_byte_for_class(i % 4, mix_index(i as u32)));
    }

    for i in (1..first_bytes.len()).rev() {
        let mixed = mix_index(i as u32);
        let j = mixed as usize % (i + 1);

        first_bytes.swap(i, j);
    }

    first_bytes
}

fn sum_first_byte_widths<F>(first_bytes: &[u8], width_of: F) -> usize
where
    F: Fn(u8) -> usize, {
    let first_bytes = black_box(first_bytes);
    let mut total_width = 0;

    for byte in first_bytes.iter().copied() {
        total_width += width_of(byte);
    }

    total_width
}

fn sum_scanned_widths<F>(bytes: &[u8], width_of: F) -> usize
where
    F: Fn(u8) -> usize, {
    let bytes = black_box(bytes);
    let length = bytes.len();
    let mut p = 0;
    let mut total_width = 0;

    while p < length {
        let width = width_of(bytes[p]);

        total_width += width;
        p += width;
    }

    total_width
}

fn sum_widths_by_chars(text: &str) -> usize {
    let text = black_box(text);
    let mut total_width = 0;

    for c in text.chars() {
        total_width += c.len_utf8();
    }

    total_width
}

fn classify_balanced_get_width(bencher: &mut Bencher) {
    let first_bytes = make_balanced_first_bytes();
    let length = first_bytes.len();

    bencher.iter(|| sum_first_byte_widths(&first_bytes, utf8_width::get_width));

    bencher.bytes = length as u64;
}

fn classify_balanced_get_width_assume_valid(bencher: &mut Bencher) {
    let first_bytes = make_balanced_first_bytes();
    let length = first_bytes.len();

    bencher.iter(|| {
        sum_first_byte_widths(&first_bytes, |byte| unsafe {
            utf8_width::get_width_assume_valid(byte)
        })
    });

    bencher.bytes = length as u64;
}

fn classify_balanced_get_width_by_looking_table(bencher: &mut Bencher) {
    let first_bytes = make_balanced_first_bytes();
    let length = first_bytes.len();

    bencher.iter(|| sum_first_byte_widths(&first_bytes, |byte| UTF8_CHAR_WIDTH[byte as usize]));

    bencher.bytes = length as u64;
}

fn scan_text_get_width(bencher: &mut Bencher) {
    let bytes = fs::read(TEXT_PATH).unwrap();
    let length = bytes.len();

    bencher.iter(|| sum_scanned_widths(&bytes, utf8_width::get_width));

    bencher.bytes = length as u64;
}

fn scan_text_get_width_assume_valid(bencher: &mut Bencher) {
    let bytes = fs::read(TEXT_PATH).unwrap();
    let length = bytes.len();

    bencher.iter(|| {
        sum_scanned_widths(&bytes, |byte| unsafe { utf8_width::get_width_assume_valid(byte) })
    });

    bencher.bytes = length as u64;
}

fn scan_text_get_width_by_looking_table(bencher: &mut Bencher) {
    let bytes = fs::read(TEXT_PATH).unwrap();
    let length = bytes.len();

    bencher.iter(|| sum_scanned_widths(&bytes, |byte| UTF8_CHAR_WIDTH[byte as usize]));

    bencher.bytes = length as u64;
}

fn scan_text_get_width_by_chars(bencher: &mut Bencher) {
    let text = fs::read_to_string(TEXT_PATH).unwrap();
    let length = text.len();

    bencher.iter(|| sum_widths_by_chars(&text));

    bencher.bytes = length as u64;
}

benchmark_group!(
    classify_balanced,
    classify_balanced_get_width,
    classify_balanced_get_width_assume_valid,
    classify_balanced_get_width_by_looking_table
);
benchmark_group!(
    scan_text,
    scan_text_get_width,
    scan_text_get_width_assume_valid,
    scan_text_get_width_by_looking_table,
    scan_text_get_width_by_chars
);
benchmark_main!(classify_balanced, scan_text);
