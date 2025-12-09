use std::io::Cursor;
use wfdb::Result;
use wfdb::signal::{Format16Decoder, FormatDecoder, INVALID_SAMPLE};

#[test]
fn test_format16_decoder() {
    // Sample data: [1, -1, 100, -32768 (invalid)]
    #[rustfmt::skip]
    let data: Vec<u8> = vec![
        0x01, 0x00,  // 1 (little-endian)
        0xFF, 0xFF,  // -1
        0x64, 0x00,  // 100
        0x00, 0x80,  // -32768 (invalid marker)
    ];

    let mut reader = Cursor::new(data);
    let mut decoder = Format16Decoder::new();

    let mut samples = vec![0; 4];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 4);
    assert_eq!(samples[0], 1);
    assert_eq!(samples[1], -1);
    assert_eq!(samples[2], 100);
    assert_eq!(samples[3], INVALID_SAMPLE);
}

#[test]
fn test_format16_decoder_partial() {
    // Only 3 bytes (incomplete sample)
    let data: Vec<u8> = vec![0x01, 0x00, 0xFF];
    let mut reader = Cursor::new(data);
    let mut decoder = Format16Decoder::new();

    let mut samples = vec![0; 4];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 1); // Only one complete sample
    assert_eq!(samples[0], 1);
}

#[test]
fn test_format16_bytes_per_sample() {
    let decoder = Format16Decoder::new();
    assert_eq!(decoder.bytes_per_sample(), Some(2));
}

#[test]
fn test_format16_decode_ergonomic() {
    // Test the ergonomic decode() API that returns Vec
    #[rustfmt::skip]
    let data: Vec<u8> = vec![
        0x01, 0x00,  // 1
        0xFF, 0xFF,  // -1
        0x64, 0x00,  // 100
    ];

    let mut reader = Cursor::new(data);
    let mut decoder = Format16Decoder::new();

    // Ergonomic API - returns Vec
    let samples = decoder.decode(&mut reader, 3).unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0], 1);
    assert_eq!(samples[1], -1);
    assert_eq!(samples[2], 100);
}

#[test]
fn test_format16_samples_iterator() {
    // Test the iterator API

    let data: Vec<u8> = vec![
        0x00, 0x00, // 0
        0x01, 0x00, // 1
        0xFF, 0xFF, // -1
        0x00, 0x80, // -32768 (WFDB_INVALID_SAMPLE)
        0x10, 0x00, // 16
    ];

    let reader = Cursor::new(data);
    let mut decoder = Format16Decoder::new();

    // Collect first 3 samples
    let first_three: Vec<_> = decoder
        .samples(reader)
        .take(3)
        .collect::<Result<Vec<_>>>()
        .unwrap();

    assert_eq!(first_three, vec![0, 1, -1]);

    // Test with filter
    let data: Vec<u8> = vec![
        0x00, 0x00, // 0
        0x01, 0x00, // 1
        0xFF, 0xFF, // -1
        0x10, 0x00, // 16
    ];

    let reader = Cursor::new(data);
    let mut decoder = Format16Decoder::new();

    // Filter positive values
    let positive: Vec<_> = decoder
        .samples(reader)
        .filter_map(Result::ok)
        .filter(|&s| s > 0)
        .collect();

    assert_eq!(positive, vec![1, 16]);
}
