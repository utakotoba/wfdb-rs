use std::io::Cursor;
use wfdb::signal::{Format0Decoder, FormatDecoder, INVALID_SAMPLE};

#[test]
fn test_format0_decoder() {
    let data: Vec<u8> = vec![];
    let mut reader = Cursor::new(data);
    let mut decoder = Format0Decoder::new();

    let mut samples = vec![0; 100];
    let n = decoder.decode_buf(&mut reader, &mut samples).unwrap();

    assert_eq!(n, 100);
    assert!(samples.iter().all(|&s| s == INVALID_SAMPLE));
}
