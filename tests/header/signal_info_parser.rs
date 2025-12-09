use wfdb::{Error, SignalFormat, header::SignalInfo};

// [Basic Parsing Tests]

#[test]
fn test_minimal_signal_line() {
    let line = "- 16";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    let expected = SignalInfo {
        file_name: "-".to_string(),
        format: SignalFormat::Format16,
        samples_per_frame: None,
        skew: None,
        byte_offset: None,
        adc_gain: None,
        baseline: None,
        units: None,
        adc_resolution: None,
        adc_zero: None,
        initial_value: None,
        checksum: None,
        block_size: None,
        description: None,
    };
    assert_eq!(signal, expected);
}

#[test]
fn test_mit_bih_signal_line() {
    let line = "100.dat 212 200 11 1024 995 -22131 0 MLII";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    let expected = SignalInfo {
        file_name: "100.dat".to_string(),
        format: SignalFormat::Format212,
        samples_per_frame: None,
        skew: None,
        byte_offset: None,
        adc_gain: Some(200.0),
        baseline: None,
        units: None,
        adc_resolution: Some(11),
        adc_zero: Some(1024),
        initial_value: Some(995),
        checksum: Some(-22131),
        block_size: Some(0),
        description: Some("MLII".to_string()),
    };
    assert_eq!(signal, expected);
}

#[test]
fn test_aha_signal_line() {
    let line = "data0 8 100 10 0 -53 -1279 0 ECG signal 0";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    let expected = SignalInfo {
        file_name: "data0".to_string(),
        format: SignalFormat::Format8,
        samples_per_frame: None,
        skew: None,
        byte_offset: None,
        adc_gain: Some(100.0),
        baseline: None,
        units: None,
        adc_resolution: Some(10),
        adc_zero: Some(0),
        initial_value: Some(-53),
        checksum: Some(-1279),
        block_size: Some(0),
        description: Some("ECG signal 0".to_string()),
    };
    assert_eq!(signal, expected);
}

#[test]
fn test_full_signal_line_with_modifiers() {
    let line = "sig.dat 16x2:100+512 200(500)/uV 12 2048 0 0 0 Channel A";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    let expected = SignalInfo {
        file_name: "sig.dat".to_string(),
        format: SignalFormat::Format16,
        samples_per_frame: Some(2),
        skew: Some(100),
        byte_offset: Some(512),
        adc_gain: Some(200.0),
        baseline: Some(500),
        units: Some("uV".to_string()),
        adc_resolution: Some(12),
        adc_zero: Some(2048),
        initial_value: Some(0),
        checksum: Some(0),
        block_size: Some(0),
        description: Some("Channel A".to_string()),
    };
    assert_eq!(signal, expected);
}

// [Format Field Variations]

#[test]
fn test_format_with_samples_per_frame() {
    let line = "data.dat 16x2";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.samples_per_frame, Some(2));
    assert_eq!(signal.skew, None);
    assert_eq!(signal.byte_offset, None);
}

#[test]
fn test_format_with_skew() {
    let line = "data.dat 16:50";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.samples_per_frame, None);
    assert_eq!(signal.skew, Some(50));
    assert_eq!(signal.byte_offset, None);
}

#[test]
fn test_format_with_byte_offset() {
    let line = "data.dat 16+1024";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.samples_per_frame, None);
    assert_eq!(signal.skew, None);
    assert_eq!(signal.byte_offset, Some(1024));
}

#[test]
fn test_format_with_samples_and_skew() {
    let line = "data.dat 16x4:25";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.samples_per_frame, Some(4));
    assert_eq!(signal.skew, Some(25));
    assert_eq!(signal.byte_offset, None);
}

#[test]
fn test_format_with_samples_and_offset() {
    let line = "data.dat 16x3+512";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.samples_per_frame, Some(3));
    assert_eq!(signal.skew, None);
    assert_eq!(signal.byte_offset, Some(512));
}

#[test]
fn test_format_with_skew_and_offset() {
    let line = "data.dat 16:10+256";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.samples_per_frame, None);
    assert_eq!(signal.skew, Some(10));
    assert_eq!(signal.byte_offset, Some(256));
}

#[test]
fn test_format_with_all_modifiers() {
    let line = "data.dat 212x2:50+1024";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format212);
    assert_eq!(signal.samples_per_frame, Some(2));
    assert_eq!(signal.skew, Some(50));
    assert_eq!(signal.byte_offset, Some(1024));
}

// [ADC Gain Field Variations]

#[test]
fn test_gain_only() {
    let line = "sig.dat 16 200";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.baseline, None);
    assert_eq!(signal.units, None);
}

#[test]
fn test_gain_with_baseline() {
    let line = "sig.dat 16 200(1024)";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.baseline, Some(1024));
    assert_eq!(signal.units, None);
}

#[test]
fn test_gain_with_units() {
    let line = "sig.dat 16 200/mV";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.baseline, None);
    assert_eq!(signal.units, Some("mV".to_string()));
}

#[test]
fn test_gain_with_baseline_and_units() {
    let line = "sig.dat 16 200(512)/uV";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.baseline, Some(512));
    assert_eq!(signal.units, Some("uV".to_string()));
}

#[test]
fn test_gain_with_negative_baseline() {
    let line = "sig.dat 16 200(-100)/mV";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.baseline, Some(-100));
    assert_eq!(signal.units, Some("mV".to_string()));
}

#[test]
fn test_floating_point_gain() {
    let line = "sig.dat 16 200.5";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert!((signal.adc_gain.unwrap() - 200.5).abs() < f64::EPSILON);
}

#[test]
fn test_exponential_notation_gain() {
    let line = "sig.dat 16 2.0e2";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert!((signal.adc_gain.unwrap() - 200.0).abs() < f64::EPSILON);
}

// [Complete ADC Specification]

#[test]
fn test_complete_adc_spec() {
    let line = "sig.dat 16 200(1024)/mV 12 2048 100 -5000 0";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.baseline, Some(1024));
    assert_eq!(signal.units, Some("mV".to_string()));
    assert_eq!(signal.adc_resolution, Some(12));
    assert_eq!(signal.adc_zero, Some(2048));
    assert_eq!(signal.initial_value, Some(100));
    assert_eq!(signal.checksum, Some(-5000));
    assert_eq!(signal.block_size, Some(0));
}

#[test]
fn test_adc_spec_with_description() {
    let line = "sig.dat 16 200 12 2048 100 -5000 0 Lead II";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.adc_resolution, Some(12));
    assert_eq!(signal.adc_zero, Some(2048));
    assert_eq!(signal.initial_value, Some(100));
    assert_eq!(signal.checksum, Some(-5000));
    assert_eq!(signal.block_size, Some(0));
    assert_eq!(signal.description, Some("Lead II".to_string()));
}

#[test]
fn test_multiword_description() {
    let line = "sig.dat 16 200 12 2048 100 0 0 Modified Lead II Channel";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(
        signal.description,
        Some("Modified Lead II Channel".to_string())
    );
}

// [File Name Variations]

#[test]
fn test_standard_input_output() {
    let line = "- 16";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.file_name, "-");
}

#[test]
fn test_relative_path() {
    let line = "data/signal.dat 16";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.file_name, "data/signal.dat");
}

#[test]
fn test_absolute_path() {
    let line = "/db1/data0/d0.7001 8";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.file_name, "/db1/data0/d0.7001");
}

#[test]
fn test_filename_with_record_name() {
    let line = "100.dat 212";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.file_name, "100.dat");
}

// [Different Format Codes]

#[test]
fn test_format_8() {
    let line = "data.dat 8";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format8);
}

#[test]
fn test_format_16() {
    let line = "data.dat 16";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format16);
}

#[test]
fn test_format_212() {
    let line = "data.dat 212";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format212);
}

#[test]
fn test_format_24() {
    let line = "data.dat 24";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.format, SignalFormat::Format24);
}

#[test]
fn test_unsupported_format_code() {
    let line = "data.dat 999";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error for unsupported format code, got {result:?}"
    );
}

// [Accessors Tests]

/// Helper function to create minimal signal info.
fn create_minimal_signal() -> SignalInfo {
    SignalInfo {
        file_name: "test.dat".to_string(),
        format: SignalFormat::Format16,
        samples_per_frame: None,
        skew: None,
        byte_offset: None,
        adc_gain: None,
        baseline: None,
        units: None,
        adc_resolution: None,
        adc_zero: None,
        initial_value: None,
        checksum: None,
        block_size: None,
        description: None,
    }
}

/// Helper function to create full signal info.
#[allow(clippy::unwrap_used)]
fn create_full_signal() -> SignalInfo {
    let line = "sig.dat 212x2:50+512 200(1024)/uV 12 2048 100 -5000 4096 ECG Lead II";
    SignalInfo::from_signal_line(line).unwrap()
}

#[test]
fn test_file_name_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.file_name(), "test.dat");
    let signal = create_full_signal();
    assert_eq!(signal.file_name(), "sig.dat");
}

#[test]
fn test_format_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.format(), SignalFormat::Format16);
    let signal = create_full_signal();
    assert_eq!(signal.format(), SignalFormat::Format212);
}

#[test]
fn test_samples_per_frame_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.samples_per_frame(), 1);
    let signal = create_full_signal();
    assert_eq!(signal.samples_per_frame(), 2);
}

#[test]
fn test_skew_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.skew(), 0);
    let signal = create_full_signal();
    assert_eq!(signal.skew(), 50);
}

#[test]
fn test_byte_offset_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.byte_offset(), 0);
    let signal = create_full_signal();
    assert_eq!(signal.byte_offset(), 512);
}

#[test]
fn test_adc_gain_accessor() {
    let signal = create_minimal_signal();
    assert!((signal.adc_gain() - 200.0).abs() < f64::EPSILON);
    let signal = create_full_signal();
    assert!((signal.adc_gain() - 200.0).abs() < f64::EPSILON);
}

#[test]
fn test_baseline_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.baseline(), 0); // defaults to adc_zero
    let signal = create_full_signal();
    assert_eq!(signal.baseline(), 1024);
}

#[test]
fn test_units_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.units(), "mV");
    let signal = create_full_signal();
    assert_eq!(signal.units(), "uV");
}

#[test]
fn test_adc_resolution_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.adc_resolution(), 12); // default for amplitude format
    let signal = create_full_signal();
    assert_eq!(signal.adc_resolution(), 12);
}

#[test]
fn test_adc_resolution_difference_format() {
    let line = "test.dat 8"; // format 8 is difference format
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.adc_resolution(), 10); // default for difference format
}

#[test]
fn test_adc_zero_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.adc_zero(), 0);
    let signal = create_full_signal();
    assert_eq!(signal.adc_zero(), 2048);
}

#[test]
fn test_initial_value_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.initial_value(), 0); // defaults to adc_zero
    let signal = create_full_signal();
    assert_eq!(signal.initial_value(), 100);
}

#[test]
fn test_checksum_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.checksum(), None);
    let signal = create_full_signal();
    assert_eq!(signal.checksum(), Some(-5000));
}

#[test]
fn test_block_size_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.block_size(), 0);
    let signal = create_full_signal();
    assert_eq!(signal.block_size(), 4096);
}

#[test]
fn test_description_accessor() {
    let signal = create_minimal_signal();
    assert_eq!(signal.description(), None);
    let signal = create_full_signal();
    assert_eq!(signal.description(), Some("ECG Lead II"));
}

// [Invalid Input Tests]

#[test]
fn test_valid_gain_format_errors() {
    // When gain is successfully parsed but invalid, it should error
    // This happens when the field format matches gain pattern but value is invalid

    // Test with integer that parses as block_size instead
    let line = "sig.dat 16 0";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.block_size, Some(0));

    // Fields that look like descriptions due to failed gain parsing
    let line = "sig.dat 16 0.0/mV";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("0.0/mV".to_string()));
}

#[test]
fn test_empty_line() {
    let result = SignalInfo::from_signal_line("");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_whitespace_only_line() {
    let result = SignalInfo::from_signal_line("   \t  ");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_missing_format() {
    let result = SignalInfo::from_signal_line("filename.dat");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_format() {
    let line = "sig.dat abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_zero_samples_per_frame() {
    let line = "sig.dat 16x0";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_samples_per_frame() {
    let line = "sig.dat 16xabc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_skew() {
    let line = "sig.dat 16:abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_byte_offset() {
    let line = "sig.dat 16+abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_zero_value_after_format() {
    // A zero after format could be block_size or description, not gain
    let line = "sig.dat 16 0";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.block_size, Some(0));
}

#[test]
fn test_negative_value_after_format() {
    // A negative value after format is parsed as block_size
    let line = "sig.dat 16 -200";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.block_size, Some(-200));
}

#[test]
fn test_non_numeric_after_format() {
    // Non-numeric after format is treated as description
    let line = "sig.dat 16 abc";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("abc".to_string()));
}

#[test]
fn test_slash_without_gain() {
    // Slash without gain is treated as description
    let line = "sig.dat 16 /mV";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("/mV".to_string()));
}

#[test]
fn test_paren_without_closing() {
    // Unclosed paren is treated as description
    let line = "sig.dat 16 200(1024";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("200(1024".to_string()));
}

#[test]
fn test_non_numeric_baseline() {
    // Non-numeric baseline is treated as description
    let line = "sig.dat 16 200(abc)";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("200(abc)".to_string()));
}

#[test]
fn test_trailing_slash() {
    // Trailing slash is treated as description
    let line = "sig.dat 16 200/";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("200/".to_string()));
}

#[test]
fn test_invalid_resolution_after_gain() {
    let line = "sig.dat 16 200 abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_adc_zero_after_resolution() {
    let line = "sig.dat 16 200 12 abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_initial_value_after_zero() {
    let line = "sig.dat 16 200 12 2048 abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_checksum_after_initial() {
    let line = "sig.dat 16 200 12 2048 100 abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_block_size_after_checksum() {
    let line = "sig.dat 16 200 12 2048 100 0 abc";
    let result = SignalInfo::from_signal_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

// [Edge Cases]

#[test]
fn test_negative_checksum() {
    let line = "sig.dat 16 200 12 2048 100 -32768 0";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.checksum, Some(-32768));
}

#[test]
fn test_positive_checksum() {
    let line = "sig.dat 16 200 12 2048 100 32767 0";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.checksum, Some(32767));
}

#[test]
fn test_negative_block_size() {
    let line = "sig.dat 16 200 12 2048 100 0 -1";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.block_size, Some(-1));
}

#[test]
fn test_large_byte_offset() {
    let line = "sig.dat 16+4294967295";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.byte_offset, Some(4_294_967_295));
}

#[test]
fn test_paren_value_without_gain() {
    // Parenthesized value without gain is treated as description
    let line = "sig.dat 16 (1024)";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("(1024)".to_string()));
}

#[test]
fn test_slash_units_without_gain() {
    // Slash with units but no gain is treated as description
    let line = "sig.dat 16 /mV";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("/mV".to_string()));
}

#[test]
fn test_whitespace_in_description() {
    // split_whitespace normalizes whitespace, so multiple spaces become single spaces
    let line = "sig.dat 16 200 12 2048 100 0 0   Multiple   Spaces   Here  ";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.description, Some("Multiple Spaces Here".to_string()));
}

#[test]
fn test_tabs_between_fields() {
    let line = "sig.dat\t16\t200\t12";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.file_name, "sig.dat");
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.adc_gain, Some(200.0));
    assert_eq!(signal.adc_resolution, Some(12));
}

#[test]
fn test_mixed_whitespace() {
    let line = "sig.dat  \t  16  \t  200";
    let signal = SignalInfo::from_signal_line(line).unwrap();
    assert_eq!(signal.file_name, "sig.dat");
    assert_eq!(signal.format, SignalFormat::Format16);
    assert_eq!(signal.adc_gain, Some(200.0));
}
