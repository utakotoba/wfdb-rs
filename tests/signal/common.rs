use wfdb::signal::sign_extend;

#[test]
fn test_sign_extend_12bit() {
    // Positive values
    assert_eq!(sign_extend(0x000, 12), 0);
    assert_eq!(sign_extend(0x7FF, 12), 2047);
    assert_eq!(sign_extend(0x001, 12), 1);

    // Negative values
    assert_eq!(sign_extend(0x800, 12), -2048);
    assert_eq!(sign_extend(0xFFF, 12), -1);
    assert_eq!(sign_extend(0xFFE, 12), -2);
}

#[test]
fn test_sign_extend_10bit() {
    // Positive values
    assert_eq!(sign_extend(0x000, 10), 0);
    assert_eq!(sign_extend(0x1FF, 10), 511);

    // Negative values
    assert_eq!(sign_extend(0x200, 10), -512);
    assert_eq!(sign_extend(0x3FF, 10), -1);
}

#[test]
fn test_sign_extend_8bit() {
    // Positive values
    assert_eq!(sign_extend(0x00, 8), 0);
    assert_eq!(sign_extend(0x7F, 8), 127);

    // Negative values
    assert_eq!(sign_extend(0x80, 8), -128);
    assert_eq!(sign_extend(0xFF, 8), -1);
}
