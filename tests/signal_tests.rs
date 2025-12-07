use std::io::Write;
use tempfile::NamedTempFile;
use wfdb::Record;

#[test]
fn test_read_format_16() {
    // Header: 2 signals, format 16, interleaved
    let header_content = "test16 2 100 10
test16.dat 16 200 0 0 0 0 0 sig1
test16.dat 16 200 0 0 0 0 0 sig2";

    // Data: 
    // Frame 0: s1=10, s2=20 -> [10, 0, 20, 0] (little endian)
    // Frame 1: s1=-10, s2=-20 -> [-10 as u16, -20 as u16] -> [246, 255, 236, 255]
    let data: [u8; 8] = [
        10, 0, 20, 0,
        246, 255, 236, 255
    ];

    let mut header_file = NamedTempFile::new().unwrap();
    write!(header_file, "{}", header_content).unwrap();
    let _header_path = header_file.path().to_path_buf();
    
    // Create .dat file in the same directory (temp dir)
    
    let dir = tempfile::tempdir().unwrap();
    let header_path = dir.path().join("test16.hea");
    let dat_path = dir.path().join("test16.dat");
    
    std::fs::write(&header_path, header_content).unwrap();
    std::fs::write(&dat_path, &data).unwrap();
    
    let mut record = Record::open(&header_path).unwrap();
    
    // Read Frame 0
    let frame0 = record.reader.read_frame().unwrap().unwrap();
    assert_eq!(frame0.len(), 2);
    assert_eq!(frame0[0], 10);
    assert_eq!(frame0[1], 20);
    
    // Read Frame 1
    let frame1 = record.reader.read_frame().unwrap().unwrap();
    assert_eq!(frame1.len(), 2);
    assert_eq!(frame1[0], -10);
    assert_eq!(frame1[1], -20);
    
    // EOF
    assert!(record.reader.read_frame().unwrap().is_none());
}

#[test]
fn test_read_format_212() {
    // Header: 2 signals, format 212
    let header_content = "test212 2 100 10
test212.dat 212 200 0 0 0 0 0 sig1
test212.dat 212 200 0 0 0 0 0 sig2";

    // Data: 2 samples per 3 bytes.
    // Frame 0: s1=10, s2=20
    // s1=0x00A, s2=0x014
    // byte0 = 0x0A
    // byte1 = (0x0 & 0xF) | (0x0 & 0xF) << 4 = 0
    // byte2 = 0x14
    // Bytes: 0x0A, 0x00, 0x14 -> 10, 0, 20
    
    // Frame 1: s1=-10, s2=-20
    // s1 = -10 = 0xFF6 (12-bit) -> 4086
    // s2 = -20 = 0xFEC (12-bit) -> 4076
    // byte0 = 0xF6
    // byte1 = (0xF & 0xF) | (0xF & 0xF) << 4 = 0xFF
    // byte2 = 0xEC
    // Bytes: 0xF6, 0xFF, 0xEC -> 246, 255, 236
    
    let data: [u8; 6] = [
        10, 0, 20,
        246, 255, 236
    ];

    let dir = tempfile::tempdir().unwrap();
    let header_path = dir.path().join("test212.hea");
    let dat_path = dir.path().join("test212.dat");
    
    std::fs::write(&header_path, header_content).unwrap();
    std::fs::write(&dat_path, &data).unwrap();
    
    let mut record = Record::open(&header_path).unwrap();
    
    // Read Frame 0
    let frame0 = record.reader.read_frame().unwrap().unwrap();
    assert_eq!(frame0[0], 10);
    assert_eq!(frame0[1], 20);
    
    // Read Frame 1
    let frame1 = record.reader.read_frame().unwrap().unwrap();
    assert_eq!(frame1[0], -10);
    assert_eq!(frame1[1], -20);
    
    // EOF
    assert!(record.reader.read_frame().unwrap().is_none());
}
