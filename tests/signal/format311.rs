use std::io::Cursor;
use wfdb::signal::{Format311Decoder, FormatDecoder};

#[test]
fn test_format311_decoder() {
    // 3 samples: 0x001 (1), 0x3FF (-1), 0x000 (0)
    // Bits 0-9: 0x001, Bits 10-19: 0x3FF, Bits 20-29: 0x000
    // As 32-bit little-endian: 0x00FFC401
    #[rustfmt::skip]
    let data: Vec<u8> = vec![
        0x01, 0xFC, 0x0F, 0x00,  // 0x000FFC01
    ];

    let mut reader = Cursor::new(data);
    let mut decoder = Format311Decoder::new();

    let mut samples = vec![0; 3];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 3);
    assert_eq!(samples[0], 1);
    assert_eq!(samples[1], -1);
    assert_eq!(samples[2], 0);
}

#[test]
fn test_format311_reset() {
    let data: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00];
    let mut reader = Cursor::new(data);
    let mut decoder = Format311Decoder::new();

    let mut samples = vec![0; 1];
    decoder.decode_buf(&mut reader, &mut samples).unwrap();

    // Reset should work without error
    decoder.reset();

    // After reset, should be able to decode again
    let data2: Vec<u8> = vec![0x01, 0xFC, 0x0F, 0x00];
    let mut reader2 = Cursor::new(data2);
    let mut samples2 = vec![0; 3];
    let n = decoder.decode_buf(&mut reader2, &mut samples2).unwrap();
    assert_eq!(n, 3);
    assert_eq!(samples2[0], 1);
    assert_eq!(samples2[1], -1);
    assert_eq!(samples2[2], 0);
}
