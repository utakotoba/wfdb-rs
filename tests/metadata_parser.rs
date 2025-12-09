use chrono::{NaiveDate, NaiveTime};
use wfdb::{Error, header::Metadata};

// [Basic Parsing Tests]

#[test]
fn test_minimal_record_line() {
    let line = "record_001 12";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "record_001".to_string(),
        num_segments: None,
        num_signals: 12,
        sampling_frequency: None,
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_full_record_line() {
    let line = "db_100/2 2 360/72(0) 650000 09:30:00 01/05/1990";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "db_100".to_string(),
        num_segments: Some(2),
        num_signals: 2,
        sampling_frequency: Some(360.0),
        counter_frequency: Some(72.0),
        base_counter: Some(0.0),
        num_samples: Some(650_000),
        base_time: Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()),
        base_date: Some(NaiveDate::from_ymd_opt(1990, 5, 1).unwrap()),
    };
    assert_eq!(metadata, expected);
}

// [Frequency Field Variations]

#[test]
fn test_with_sampling_frequency_only() {
    let line = "rec 4 500";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 4,
        sampling_frequency: Some(500.0),
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_with_floating_point_frequency() {
    let line = "rec 2 360.5";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(360.5),
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_with_decimal_point_frequency() {
    let line = "rec 2 360.";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(360.0),
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_with_exponential_notation_frequency() {
    let line = "rec 2 3.6e2";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(360.0),
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_with_counter_frequency_no_base() {
    let line = "rec 4 500/100";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 4,
        sampling_frequency: Some(500.0),
        counter_frequency: Some(100.0),
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_with_decimal_point_counter_frequency() {
    let line = "rec 2 500/100.";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: Some(100.0),
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_with_exponential_notation_counter_frequency() {
    let line = "rec 2 500/1.0e2";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: Some(100.0),
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_with_counter_frequency_and_base() {
    let line = "rec 2 500/100(50)";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: Some(100.0),
        base_counter: Some(50.0),
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

// [Optional Field Detection (Omitted Fields in the Middle)]

#[test]
fn test_time_only() {
    let line = "rec 2 09:15:30";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: None,
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: Some(NaiveTime::from_hms_opt(9, 15, 30).unwrap()),
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_time_and_date_only() {
    let line = "rec 2 12:30:45 01/01/2000";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: None,
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: Some(NaiveTime::from_hms_opt(12, 30, 45).unwrap()),
        base_date: Some(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_frequency_and_time_no_samples() {
    let line = "rec 2 500 12:30:45";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: Some(NaiveTime::from_hms_opt(12, 30, 45).unwrap()),
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_samples_and_time_no_frequency() {
    let line = "rec 2 650000 12:30:45";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(650_000.0), // first numeric is always frequency
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: Some(NaiveTime::from_hms_opt(12, 30, 45).unwrap()),
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_zero_counter_frequency() {
    let line = "rec 2 500/0";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: None, // when counter frequency is 0, it is omitted
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_zero_counter_frequency_with_base() {
    let line = "rec 2 500/0(100)";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: None, // when counter frequency is 0, it is omitted
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_negative_counter_frequency() {
    let line = "rec 2 500/-100";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: None, // when counter frequency is negative, it is omitted
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

#[test]
fn test_negative_counter_frequency_with_base() {
    let line = "rec 2 500/-100(100)";
    let metadata = Metadata::from_record_line(line).unwrap();
    let expected = Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: Some(500.0),
        counter_frequency: None, // when counter frequency is negative, it is omitted
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    };
    assert_eq!(metadata, expected);
}

// [Accessors Tests]

/// Helper function to create minimal metadata.
fn create_minimal_metadata() -> Metadata {
    Metadata {
        name: "rec".to_string(),
        num_segments: None,
        num_signals: 2,
        sampling_frequency: None,
        counter_frequency: None,
        base_counter: None,
        num_samples: None,
        base_time: None,
        base_date: None,
    }
}

/// Helper function to create full metadata.
#[allow(clippy::unwrap_used)]
fn create_full_metadata() -> Metadata {
    let line = "db_100/2 4 360/72(50) 650000 09:30:00 01/05/1990";
    Metadata::from_record_line(line).unwrap()
}

#[test]
fn test_name_accessor() {
    let metadata = create_minimal_metadata();
    assert_eq!(metadata.name(), "rec");
    let metadata = create_full_metadata();
    assert_eq!(metadata.name(), "db_100");
}

#[test]
fn test_num_segments_accessor() {
    let metadata = create_minimal_metadata();
    assert_eq!(metadata.num_segments(), None);
    let metadata = create_full_metadata();
    assert_eq!(metadata.num_segments(), Some(2));
}

#[test]
fn test_num_signals_accessor() {
    let metadata = create_minimal_metadata();
    assert_eq!(metadata.num_signals(), 2);
    let metadata = create_full_metadata();
    assert_eq!(metadata.num_signals(), 4);
}

#[test]
fn test_sampling_frequency_accessor() {
    let metadata = create_minimal_metadata();
    assert!((metadata.sampling_frequency() - 250.0).abs() < f64::EPSILON);
    let metadata = create_full_metadata();
    assert!((metadata.sampling_frequency() - 360.0).abs() < f64::EPSILON);
}

#[test]
fn test_counter_frequency_accessor() {
    let metadata = create_minimal_metadata();
    assert!((metadata.counter_frequency() - 250.0).abs() < f64::EPSILON);
    let metadata = create_full_metadata();
    assert!((metadata.counter_frequency() - 72.0).abs() < f64::EPSILON);
}

#[test]
fn test_base_counter_accessor() {
    let metadata = create_minimal_metadata();
    assert!((metadata.base_counter() - 0.0).abs() < f64::EPSILON);
    let metadata = create_full_metadata();
    assert!((metadata.base_counter() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_num_samples_accessor() {
    let metadata = create_minimal_metadata();
    assert_eq!(metadata.num_samples(), None);
    let metadata = create_full_metadata();
    assert_eq!(metadata.num_samples(), Some(650_000));
}

#[test]
fn test_base_time_accessor() {
    let metadata = create_minimal_metadata();
    assert_eq!(metadata.base_time(), None);
    let metadata = create_full_metadata();
    assert_eq!(
        metadata.base_time(),
        Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap())
    );
}

#[test]
fn test_base_date_accessor() {
    let metadata = create_minimal_metadata();
    assert_eq!(metadata.base_date(), None);
    let metadata = create_full_metadata();
    assert_eq!(
        metadata.base_date(),
        Some(NaiveDate::from_ymd_opt(1990, 5, 1).unwrap())
    );
}

// [Field Order Validation]

#[test]
fn test_duplicate_time_field() {
    let line = "rec 2 12:30:45 13:00:00";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error for duplicate time, got {result:? }"
    );
}

#[test]
fn test_duplicate_date_field() {
    let line = "rec 2 12:30:45 01/01/2000 02/02/2001";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error for duplicate date, got {result:?}"
    );
}

#[test]
fn test_date_before_time() {
    let line = "rec 2 01/01/2000 12:30:45";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error for out-of-order fields, got {result:?}"
    );
}

#[test]
fn test_numeric_after_time() {
    let line = "rec 2 12:30:45 500";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error for numeric after time, got {result:?}"
    );
}

#[test]
fn test_numeric_after_date() {
    let line = "rec 2 01/01/2000 500";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error for numeric after date, got {result:?}"
    );
}

#[test]
fn test_duplicate_frequency_field() {
    let line = "rec 2 500/100 600/200";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error for duplicate frequency, got {result:?}"
    );
}

// [Invalid Input Tests]

#[test]
fn test_empty_line() {
    let result = Metadata::from_record_line("");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:? }"
    );
}

#[test]
fn test_whitespace_only_line() {
    let result = Metadata::from_record_line("   \t  ");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:? }"
    );
}

#[test]
fn test_zero_num_segments() {
    let line = "rec/0 2";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_zero_num_signals() {
    let line = "rec 0";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_missing_num_signals() {
    let result = Metadata::from_record_line("record");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_record_name() {
    let result = Metadata::from_record_line("record-name 2");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_empty_record_name() {
    let result = Metadata::from_record_line("/2 4");
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_num_signals() {
    let line = "rec abc";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_sampling_frequency() {
    let line = "rec 2 abc";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_negative_sampling_frequency() {
    let line = "rec 2 -100";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_zero_sampling_frequency() {
    let line = "rec 2 0";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_counter_frequency() {
    let line = "rec 2 500/abc";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_counter_frequency_with_base() {
    let line = "rec 2 500/abc(100)";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_counter_frequency_without_sampling_frequency() {
    let line = "rec 2 /100(100)";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_base_counter() {
    let line = "rec 2 500/100(abc)";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_base_counter_only() {
    let line = "rec 2 (100)";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_num_segments() {
    let line = "rec/abc 2";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_missing_closing_paren() {
    let line = "rec 2 500/100(50";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_num_samples() {
    let line = "rec 2 100 -1.5";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_time_format() {
    let line = "rec 2 25:00:00";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_invalid_date_format() {
    let line = "rec 2 12:30:45 32//20";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}

#[test]
fn test_date_only_without_time() {
    let line = "rec 2 01/01/2000";
    let result = Metadata::from_record_line(line);
    assert!(
        matches!(result, Err(Error::InvalidHeader(_))),
        "Expected InvalidHeader error, got {result:?}"
    );
}
