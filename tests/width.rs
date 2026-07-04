use utf8_width::{get_width, is_width_0, is_width_1, is_width_2, is_width_3, is_width_4};

#[test]
fn get_width_returns_expected_width_for_valid_first_byte_boundaries() {
    let cases =
        [(0x00, 1), (0x7F, 1), (0xC2, 2), (0xDF, 2), (0xE0, 3), (0xEF, 3), (0xF0, 4), (0xF4, 4)];

    for (byte, width) in cases.iter().copied() {
        assert_eq!(width, get_width(byte));
    }
}

#[test]
fn get_width_returns_zero_for_invalid_first_byte_boundaries() {
    let cases = [0x80, 0xBF, 0xC0, 0xC1, 0xF5, 0xFF];

    for byte in cases.iter().copied() {
        assert_eq!(0, get_width(byte));
    }
}

#[test]
fn width_predicates_classify_boundary_bytes() {
    let cases = [
        (0x00, 1),
        (0x7F, 1),
        (0x80, 0),
        (0xBF, 0),
        (0xC0, 0),
        (0xC1, 0),
        (0xC2, 2),
        (0xDF, 2),
        (0xE0, 3),
        (0xEF, 3),
        (0xF0, 4),
        (0xF4, 4),
        (0xF5, 0),
        (0xFF, 0),
    ];

    for (byte, width) in cases.iter().copied() {
        assert_eq!(width == 0, is_width_0(byte));
        assert_eq!(width == 1, is_width_1(byte));
        assert_eq!(width == 2, is_width_2(byte));
        assert_eq!(width == 3, is_width_3(byte));
        assert_eq!(width == 4, is_width_4(byte));
    }
}
