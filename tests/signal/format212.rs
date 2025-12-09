use std::io::Cursor;
use wfdb::signal::{Format212Decoder, FormatDecoder, INVALID_SAMPLE};

#[test]
fn test_format212_decoder() {
    // Two samples: 0x001 (1) and 0x7FF (2047)
    // Byte 0: 0x01 (low 8 bits of sample 0)
    // Byte 1: 0xF0 (high 4 bits of sample 0 = 0x0, high 4 bits of sample 1 = 0xF)
    // Byte 2: 0xFF (low 8 bits of sample 1)
    // Sample 0 = 0x001 = 1
    // Sample 1 = 0xFFF = -1 (in 12-bit two's complement)

    #[rustfmt::skip]
    let data: Vec<u8> = vec![
        0x01, 0xF0,  0xFF,  // Sample 0: 0x001 (1), Sample 1: 0xFFF (-1)
        0xFF, 0x07,  0x00,  // Sample 0: 0x7FF (2047), Sample 1: 0x000 (0)
    ];

    let mut reader = Cursor::new(data);
    let mut decoder = Format212Decoder::new();

    let mut samples = vec![0; 4];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 4);
    assert_eq!(samples[0], 1);
    assert_eq!(samples[1], -1);
    assert_eq!(samples[2], 2047);
    assert_eq!(samples[3], 0);
}

#[test]
fn test_format212_invalid_marker() {
    // Sample with invalid marker: 0x800 (-2048)
    // Byte 0: 0x00
    // Byte 1: 0x08 (high 4 bits of sample 0 = 0x8)
    // Byte 2: 0x00
    #[rustfmt::skip]
    let data: Vec<u8> = vec![
        0x00, 0x08, 0x00,  // Sample 0: 0x800 (-2048, invalid)
    ];

    let mut reader = Cursor::new(data);
    let mut decoder = Format212Decoder::new();

    let mut samples = vec![0; 2];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 2);
    assert_eq!(samples[0], INVALID_SAMPLE);
    assert_eq!(samples[1], 0);
}

#[test]
fn test_format212_reset() {
    let data: Vec<u8> = vec![0x01, 0x00, 0x00];
    let mut reader = Cursor::new(data);
    let mut decoder = Format212Decoder::new();

    let mut samples = vec![0; 2];
    decoder.decode_buf(&mut reader, &mut samples).unwrap();

    // Reset should work without error
    decoder.reset();

    // After reset, should be able to decode again from beginning
    let data2: Vec<u8> = vec![0x01, 0xF0, 0xFF];
    let mut reader2 = Cursor::new(data2);
    let mut samples2 = vec![0; 2];
    let n = decoder.decode_buf(&mut reader2, &mut samples2).unwrap();
    assert_eq!(n, 2);
    assert_eq!(samples2[0], 1);
    assert_eq!(samples2[1], -1);
}

#[test]
fn test_format212_decode_ergonomic() {
    // Test the ergonomic decode() API
    #[rustfmt::skip]
    let data: Vec<u8> = vec![
        0x01, 0xF0,  0xFF,  // Sample 0: 0x001 (1), Sample 1: 0xFFF (-1)
    ];

    let mut reader = Cursor::new(data);
    let mut decoder = Format212Decoder::new();

    // Ergonomic API - returns Vec
    let samples = decoder.decode(&mut reader, 2).unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0], 1);
    assert_eq!(samples[1], -1);
}
