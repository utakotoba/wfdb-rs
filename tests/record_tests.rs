use std::io::Cursor;
use wfdb::{Header, Record};

#[test]
fn test_record_open_from_memory() {
    let header_text = "100 2 360 650000\n\
                      100.dat 212 200 11 1024 995 43405 0 MLII\n\
                      100.dat 212 200 11 1024 1011 20052 0 V5\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    let record = Record::from_header(header, ".".into());

    assert_eq!(record.metadata().name(), "100");
    assert!(!record.is_multi_segment());
    assert_eq!(record.signal_count(), 2);
    assert_eq!(record.segment_count(), 0);

    let signals = record.signal_info().unwrap();
    assert_eq!(signals[0].file_name, "100.dat");
    assert_eq!(signals[0].description, Some("MLII".to_string()));
    assert_eq!(signals[1].description, Some("V5".to_string()));
}

#[test]
fn test_record_multi_segment() {
    let header_text = "multi/3 2 360 45000\n\
                      100s 21600\n\
                      ~ 1800\n\
                      100s 21600\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();

    let record = Record::from_header(header, "multi".into());

    assert!(record.is_multi_segment());
    assert_eq!(record.segment_count(), 3);
    assert_eq!(record.signal_count(), 0);

    let segments = record.segment_info().unwrap();
    assert_eq!(segments[0].record_name, "100s");
    assert_eq!(segments[0].num_samples, 21600);
    assert_eq!(segments[1].record_name, "~");
    assert_eq!(segments[1].num_samples, 1800);
    assert_eq!(segments[2].record_name, "100s");
}

#[test]
fn test_record_accessors() {
    let header_text = "test 1 250 1000\n\
                      test.dat 16 200 12 0 0 0 0 ECG\n";

    let mut reader = Cursor::new(header_text);
    let header = Header::from_reader(&mut reader).unwrap();
    let record = Record::from_header(header, ".".into());

    assert_eq!(record.metadata().name(), "test");
    assert!((record.metadata().sampling_frequency() - 250.0).abs() < f64::EPSILON);
    assert_eq!(record.metadata().num_samples, Some(1000));

    let signals = record.signal_info().unwrap();
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].file_name, "test.dat");
}

// Note: Iterator functionality is tested through integration tests
// in tests/signal_tests.rs that read actual signal files.
