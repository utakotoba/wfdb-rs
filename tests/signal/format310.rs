use std::io::Cursor;
use wfdb::signal::{Format310Decoder, FormatDecoder};

#[test]
fn test_format310_reset() {
    let data: Vec<u8> = vec![0x02, 0x00, 0x00, 0x00];
    let mut reader = Cursor::new(data);
    let mut decoder = Format310Decoder::new();

    let mut samples = vec![0; 1];
    decoder.decode_buf(&mut reader, &mut samples).unwrap();

    // Reset should work without error
    decoder.reset();

    // After reset, should be able to decode again
    let data2: Vec<u8> = vec![0x02, 0x00, 0x00, 0x00];
    let mut reader2 = Cursor::new(data2);
    let mut samples2 = vec![0; 1];
    let n = decoder.decode_buf(&mut reader2, &mut samples2).unwrap();
    assert_eq!(n, 1);
}
