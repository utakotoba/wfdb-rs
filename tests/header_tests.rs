use std::io::Write;
use tempfile::NamedTempFile;
use wfdb::SignalFormat;
use wfdb::header::parse_header;

#[test]
fn test_parse_simple_header() {
    let header_content = "100 2 360 650000 12:00:00 01/01/2000
100.dat 212 200 11 1024 995 0 MLII
100.dat 212 200 11 1024 995 0 V5";

    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", header_content).unwrap();

    let header = parse_header(file.path()).unwrap();

    assert_eq!(header.metadata.name, "100");
    assert_eq!(header.metadata.num_signals, 2);
    assert_eq!(header.metadata.sampling_frequency, 360.0);
    assert_eq!(header.metadata.num_samples, Some(650000));
    assert_eq!(header.metadata.base_time, Some("12:00:00".to_string()));
    assert_eq!(header.metadata.base_date, Some("01/01/2000".to_string()));

    assert_eq!(header.signals.len(), 2);

    let sig0 = &header.signals[0];
    assert_eq!(sig0.file_name, "100.dat");
    assert_eq!(sig0.format, SignalFormat::Format212);
    assert_eq!(sig0.gain, 200.0);
    assert_eq!(sig0.adc_res, 11);
    assert_eq!(sig0.adc_zero, 1024);
    assert_eq!(sig0.init_value, 995);
    assert_eq!(sig0.checksum, 0);
    assert_eq!(sig0.description, Some("MLII".to_string()));
}

#[test]
fn test_parse_multi_segment_header() {
    let header_content = "multi/2 2 360 650000
100s 21600
null 21600";

    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", header_content).unwrap();

    let header = parse_header(file.path()).unwrap();

    assert_eq!(header.metadata.name, "multi");
    assert!(header.segments.is_some());

    let segments = header.segments.as_ref().unwrap();
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].name, "100s");
    assert_eq!(segments[0].num_samples, 21600);
    assert_eq!(segments[1].name, "null");
    assert_eq!(segments[1].num_samples, 21600);
}

#[test]
fn test_parse_complex_header() {
    let header_content = "100 12 100 1000
100.dat 16 1000.0(0)/mV 16 0 -119 1508 0 I
100.dat 16 1000.0(0)/mV 16 0 -55 723 0 II
100.dat 16 1000.0(0)/mV 16 0 64 64758 0 III
100.dat 16 1000.0(0)/mV 16 0 86 64423 0 AVR
100.dat 16 1000.0(0)/mV 16 0 -91 1211 0 AVL
100.dat 16 1000.0(0)/mV 16 0 4 7 0 AVF
100.dat 16 1000.0(0)/mV 16 0 -69 63827 0 V1
100.dat 16 1000.0(0)/mV 16 0 -31 6999 0 V2
100.dat 16 1000.0(0)/mV 16 0 0 63759 0 V3
100.dat 16 1000.0(0)/mV 16 0 -26 61447 0 V4
100.dat 16 1000.0(0)/mV 16 0 -39 64979 0 V5
100.dat 16 1000.0(0)/mV 16 0 -79 832 0 V6";

    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", header_content).unwrap();

    let header = parse_header(file.path()).unwrap();

    assert_eq!(header.metadata.name, "100");
    assert_eq!(header.metadata.num_signals, 12);
    assert_eq!(header.metadata.sampling_frequency, 100.0);
    assert_eq!(header.metadata.num_samples, Some(1000));

    assert_eq!(header.signals.len(), 12);

    let sig0 = &header.signals[0];
    assert_eq!(sig0.file_name, "100.dat");
    assert_eq!(sig0.format, SignalFormat::Format16);
    assert_eq!(sig0.gain, 1000.0);
    assert_eq!(sig0.baseline, 0);
    assert_eq!(sig0.units, "mV");
    assert_eq!(sig0.adc_res, 16);
    assert_eq!(sig0.adc_zero, 0);
    assert_eq!(sig0.init_value, -119);
    assert_eq!(sig0.checksum, 1508);
    assert_eq!(sig0.block_size, 0);
    assert_eq!(sig0.description, Some("I".to_string()));

    let sig6 = &header.signals[6];
    assert_eq!(sig6.description, Some("V1".to_string()));
    assert_eq!(sig6.init_value, -69);
    assert_eq!(sig6.checksum, 63827);
}
