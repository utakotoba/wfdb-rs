use std::io::Cursor;
use wfdb::signal::{Format8Decoder, FormatDecoder};

#[test]
fn test_format8_decoder() {
    // Initial value: 100
    // Differences: +10, -5, +3
    // Expected samples: 110, 105, 108
    let data: Vec<u8> = vec![10, 251, 3]; // 251 = -5 as u8

    let mut reader = Cursor::new(data);
    let mut decoder = Format8Decoder::new(100);

    let mut samples = vec![0; 3];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 3);
    assert_eq!(samples[0], 110);
    assert_eq!(samples[1], 105);
    assert_eq!(samples[2], 108);
}

#[test]
fn test_format8_saturation() {
    // Test that large differences are handled with saturation
    let data: Vec<u8> = vec![127, 127, 127]; // Max positive differences

    let mut reader = Cursor::new(data);
    let mut decoder = Format8Decoder::new(0);

    let mut samples = vec![0; 3];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 3);
    assert_eq!(samples[0], 127);
    assert_eq!(samples[1], 254);
    assert_eq!(samples[2], 381);
}

#[test]
fn test_format8_reset() {
    let data: Vec<u8> = vec![10];
    let mut reader = Cursor::new(data);
    let mut decoder = Format8Decoder::new(100);

    let mut samples = vec![0; 1];
    decoder.decode_buf(&mut reader, &mut samples).unwrap();

    // After decoding, reset should work without error
    decoder.reset();

    // After reset, decoding should start fresh
    let data2: Vec<u8> = vec![10];
    let mut reader2 = Cursor::new(data2);
    let mut samples2 = vec![0; 1];
    decoder.decode_buf(&mut reader2, &mut samples2).unwrap();

    // After reset, should decode from 0, not from 110
    assert_eq!(samples2[0], 10);
}

#[test]
fn test_format8_bytes_per_sample() {
    let decoder = Format8Decoder::new(0);
    assert_eq!(decoder.bytes_per_sample(), Some(1));
}
