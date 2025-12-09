use std::io::Cursor;
use wfdb::{Error, Header};

// [Basic Parsing Tests]

#[test]
fn test_single_segment_record() {
    let header_text = "100 2 360 650000\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n\
                      # Info string 1\n\
                      # Info string 2\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "100");
    assert_eq!(header.metadata().num_signals(), 2);
    assert!(!header.is_multi_segment());
    assert_eq!(header.signals().unwrap().len(), 2);
    assert!(header.segments().is_none());
    assert_eq!(header.info_strings().len(), 2);
    assert_eq!(header.info_strings()[0], " Info string 1");
    assert_eq!(header.info_strings()[1], " Info string 2");
}

#[test]
fn test_multi_segment_record() {
    let header_text = "multi/3 2 360 45000\n\
                      100s 21600\n\
                      null 1800\n\
                      100s 21600\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "multi");
    assert_eq!(header.metadata().num_signals(), 2);
    assert!(header.is_multi_segment());
    assert_eq!(header.num_segments(), Some(3));
    assert!(header.signals().is_none());
    assert_eq!(header.segments().unwrap().len(), 3);
    assert_eq!(header.info_strings().len(), 0);
}

#[test]
fn test_with_leading_comments() {
    let header_text = "# Comment 1\n\
                      # Comment 2\n\
                      100 2 360\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "100");
    assert_eq!(header.signals().unwrap().len(), 2);
}

#[test]
fn test_empty_lines_before_record() {
    let header_text = "\n\n100 2 360\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "100");
    assert_eq!(header.signals().unwrap().len(), 2);
}

#[test]
fn test_mixed_comments_and_empty_lines() {
    let header_text = "# Comment 1\n\
                      \n\
                      # Comment 2\n\
                      100 2 360\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "100");
}

// [Info Strings Tests]

#[test]
fn test_info_strings_without_leading_space() {
    let header_text = "100 2 360\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n\
                      #Info1\n\
                      #Info2\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.info_strings().len(), 2);
    assert_eq!(header.info_strings()[0], "Info1");
    assert_eq!(header.info_strings()[1], "Info2");
}

#[test]
fn test_multiple_info_strings() {
    let header_text = "100 2 360\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n\
                      # Line 1\n\
                      # Line 2\n\
                      # Line 3\n\
                      # Line 4\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.info_strings().len(), 4);
}

#[test]
fn test_no_info_strings() {
    let header_text = "100 2 360\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.info_strings().len(), 0);
}

// [Multi-Segment Record Tests]

#[test]
fn test_multi_segment_with_null_segment() {
    let header_text = "rec/3 2 360 45000\n\
                      seg1 21600\n\
                      ~ 1800\n\
                      seg2 21600\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert!(header.is_multi_segment());
    let segments = header.segments().unwrap();
    assert_eq!(segments.len(), 3);
    assert_eq!(segments[0].record_name(), "seg1");
    assert_eq!(segments[1].record_name(), "~");
    assert!(segments[1].is_null_segment());
    assert_eq!(segments[2].record_name(), "seg2");
}

#[test]
fn test_multi_segment_single_segment() {
    let header_text = "rec/1 2 360 21600\n\
                      seg1 21600\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert!(header.is_multi_segment());
    assert_eq!(header.num_segments(), Some(1));
    assert_eq!(header.segments().unwrap().len(), 1);
}

// [Error Cases]

#[test]
fn test_missing_record_line() {
    let header_text = "# Only comments\n\
                      # No record line\n";

    let mut reader = Cursor::new(header_text);
    let result = Header::from_reader(&mut reader);

    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Missing record line"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_insufficient_signal_specifications() {
    let header_text = "100 2 360\n\
                      100.dat 212 200\n";

    let mut reader = Cursor::new(header_text);
    let result = Header::from_reader(&mut reader);

    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Expected 2 signal specifications"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_insufficient_segment_specifications() {
    let header_text = "multi/3 2 360\n\
                      100s 21600\n";

    let mut reader = Cursor::new(header_text);
    let result = Header::from_reader(&mut reader);

    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Expected 3 segment specifications"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_empty_header() {
    let header_text = "";

    let mut reader = Cursor::new(header_text);
    let result = Header::from_reader(&mut reader);

    assert!(result.is_err());
}

#[test]
fn test_only_whitespace() {
    let header_text = "   \n\t\n   ";

    let mut reader = Cursor::new(header_text);
    let result = Header::from_reader(&mut reader);

    assert!(result.is_err());
}

// [Accessor Tests]

#[test]
fn test_metadata_accessor() {
    let header_text = "100 2 360 650000\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    let metadata = header.metadata();
    assert_eq!(metadata.name(), "100");
    assert_eq!(metadata.num_signals(), 2);
    assert!((metadata.sampling_frequency() - 360.0).abs() < f64::EPSILON);
}

#[test]
fn test_num_signals_accessor() {
    let header_text = "100 3 360\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n\
                      100.dat 212 200\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.num_signals(), 3);
}

// [Real-World Examples]

#[test]
fn test_mit_bih_style_header() {
    let header_text = "100 2 360 650000\n\
                      100.dat 212 200 11 1024 995 43405 0 MLII\n\
                      100.dat 212 200 11 1024 1011 20052 0 V5\n\
                      # 69 M 1085 1629 x1\n\
                      # Aldomet, Inderal\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "100");
    assert_eq!(header.num_signals(), 2);
    assert!(!header.is_multi_segment());

    let signals = header.signals().unwrap();
    assert_eq!(signals[0].description(), Some("MLII"));
    assert_eq!(signals[1].description(), Some("V5"));

    assert_eq!(header.info_strings().len(), 2);
}

#[test]
fn test_aha_db_style_header() {
    let header_text = "7001 2 250 525000\n\
                      data0 8 100 10 0 -53 64257 0 ECG signal 0\n\
                      data1 8 100 10 0 -69 15626 0 ECG signal 1\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "7001");
    assert!((header.metadata().sampling_frequency() - 250.0).abs() < f64::EPSILON);
    assert_eq!(header.signals().unwrap().len(), 2);
}

// [Edge Cases]

#[test]
fn test_carriage_return_line_endings() {
    let header_text = "100 2 360\r\n100.dat 212 200\r\n100.dat 212 200\r\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "100");
    assert_eq!(header.signals().unwrap().len(), 2);
}

#[test]
fn test_mixed_line_endings() {
    let header_text = "100 2 360\r\n100.dat 212 200\n100.dat 212 200\r\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    assert_eq!(header.metadata().name(), "100");
    assert_eq!(header.signals().unwrap().len(), 2);
}
