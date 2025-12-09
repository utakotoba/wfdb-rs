use wfdb::{Error, SegmentInfo};

// [Basic Parsing Tests]

#[test]
fn test_simple_segment_line() {
    let line = "100s 21600";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "100s".to_string(),
        num_samples: 21600,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_null_segment() {
    let line = "~ 1800";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "~".to_string(),
        num_samples: 1800,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_null_segment_alternative_name() {
    let line = "null 1800";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "null".to_string(),
        num_samples: 1800,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_segment_with_large_sample_count() {
    let line = "ecg_data 360000";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "ecg_data".to_string(),
        num_samples: 360_000,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_segment_with_underscore() {
    let line = "segment_01 5000";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "segment_01".to_string(),
        num_samples: 5000,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_segment_with_numeric_name() {
    let line = "123456 10000";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "123456".to_string(),
        num_samples: 10000,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_segment_with_mixed_alphanumeric() {
    let line = "rec001_seg2 250000";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "rec001_seg2".to_string(),
        num_samples: 250_000,
    };
    assert_eq!(segment, expected);
}

// [Sample Count Variations]

#[test]
fn test_zero_samples() {
    let line = "layout 0";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "layout".to_string(),
        num_samples: 0,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_single_sample() {
    let line = "short 1";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    let expected = SegmentInfo {
        record_name: "short".to_string(),
        num_samples: 1,
    };
    assert_eq!(segment, expected);
}

#[test]
fn test_very_large_sample_count() {
    let line = "longrecord 18446744073709551615";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.num_samples, 18_446_744_073_709_551_615_u64);
}

// [Whitespace Handling]

#[test]
fn test_with_leading_whitespace() {
    let line = "   100s 21600";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "100s");
    assert_eq!(segment.num_samples, 21600);
}

#[test]
fn test_with_trailing_whitespace() {
    let line = "100s 21600   ";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "100s");
    assert_eq!(segment.num_samples, 21600);
}

#[test]
fn test_with_multiple_spaces() {
    let line = "100s     21600";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "100s");
    assert_eq!(segment.num_samples, 21600);
}

#[test]
fn test_with_tabs() {
    let line = "100s\t21600";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "100s");
    assert_eq!(segment.num_samples, 21600);
}

// [Accessors]

#[test]
fn test_accessor_record_name() {
    let segment = SegmentInfo {
        record_name: "test_segment".to_string(),
        num_samples: 5000,
    };
    assert_eq!(segment.record_name(), "test_segment");
}

#[test]
fn test_accessor_num_samples() {
    let segment = SegmentInfo {
        record_name: "test_segment".to_string(),
        num_samples: 5000,
    };
    assert_eq!(segment.num_samples(), 5000);
}

#[test]
fn test_is_null_segment_true() {
    let segment = SegmentInfo {
        record_name: "~".to_string(),
        num_samples: 1800,
    };
    assert!(segment.is_null_segment());
}

#[test]
fn test_is_null_segment_false() {
    let segment = SegmentInfo {
        record_name: "100s".to_string(),
        num_samples: 21600,
    };
    assert!(!segment.is_null_segment());
}

#[test]
fn test_is_null_segment_false_for_null_name() {
    let segment = SegmentInfo {
        record_name: "null".to_string(),
        num_samples: 1800,
    };
    assert!(!segment.is_null_segment());
}

// [Error Cases]

#[test]
fn test_empty_line() {
    let line = "";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Missing record name"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_missing_num_samples() {
    let line = "100s";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Missing number of samples"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_invalid_num_samples_not_a_number() {
    let line = "100s abc";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Invalid number of samples"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_invalid_num_samples_negative() {
    let line = "100s -100";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Invalid number of samples"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_invalid_num_samples_floating_point() {
    let line = "100s 21600.5";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Invalid number of samples"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_extra_fields() {
    let line = "100s 21600 extra";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Extra fields"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_multiple_extra_fields() {
    let line = "100s 21600 field1 field2 field3";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Extra fields"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_invalid_record_name_with_space() {
    let line = "rec ord 1000";
    // This should parse "rec" as the record name and "ord" as num_samples,
    // which will fail because "ord" is not a valid number
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
}

#[test]
fn test_invalid_record_name_with_special_chars() {
    let line = "rec@rd 1000";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Invalid record name"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_invalid_record_name_with_hyphen() {
    let line = "rec-001 1000";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Invalid record name"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_invalid_record_name_with_slash() {
    let line = "rec/001 1000";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Invalid record name"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_invalid_record_name_with_period() {
    let line = "rec.001 1000";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Invalid record name"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

// [Edge Cases]

#[test]
fn test_only_whitespace() {
    let line = "   \t   ";
    let result = SegmentInfo::from_segment_line(line);
    assert!(result.is_err());
    if let Err(Error::InvalidHeader(msg)) = result {
        assert!(msg.contains("Missing record name"));
    } else {
        panic!("Expected InvalidHeader error");
    }
}

#[test]
fn test_single_character_record_name() {
    let line = "a 100";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "a");
    assert_eq!(segment.num_samples, 100);
}

#[test]
fn test_very_long_record_name() {
    let name = "a".repeat(100);
    let line = format!("{name} 1000");
    let segment = SegmentInfo::from_segment_line(&line).unwrap();
    assert_eq!(segment.record_name, name);
    assert_eq!(segment.num_samples, 1000);
}

// [Real-World Examples from Documentation]

#[test]
fn test_mit_db_example_segment_1() {
    // From example 6 in the documentation
    let line = "100s 21600";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "100s");
    assert_eq!(segment.num_samples, 21600);
}

#[test]
fn test_mit_db_example_segment_2_null() {
    // From example 6 in the documentation
    let line = "null 1800";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "null");
    assert_eq!(segment.num_samples, 1800);
}

#[test]
fn test_mit_db_example_segment_3() {
    // From example 6 in the documentation
    let line = "100s 21600";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "100s");
    assert_eq!(segment.num_samples, 21600);
}

// [Layout Segment (Zero Samples)]

#[test]
fn test_layout_segment_zero_samples() {
    let line = "layout_segment 0";
    let segment = SegmentInfo::from_segment_line(line).unwrap();
    assert_eq!(segment.record_name, "layout_segment");
    assert_eq!(segment.num_samples, 0);
    assert!(!segment.is_null_segment());
}
