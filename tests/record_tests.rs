use wfdb::open;

#[test]
fn test_open_single_record() {
    let header_content = "rec1 1 100 10\nrec1.dat 16 200 0 0 0 0 0 sig1";
    let data: [u8; 20] = [0; 20]; // 10 samples * 2 bytes = 20 bytes

    let dir = tempfile::tempdir().unwrap();
    let header_path = dir.path().join("rec1.hea");
    let dat_path = dir.path().join("rec1.dat");
    
    std::fs::write(&header_path, header_content).unwrap();
    std::fs::write(&dat_path, &data).unwrap();
    
    // Open using the header path
    let mut iterator = open(&header_path).unwrap();
    let record = iterator.next().unwrap().unwrap();
    assert_eq!(record.header.metadata.name, "rec1");
    assert!(iterator.next().is_none());
}

#[test]
fn test_open_directory() {
    let dir = tempfile::tempdir().unwrap();
    
    // Record 1
    let h1 = "r1 1 100 10\nr1.dat 16 200 0 0 0 0 0 s1";
    std::fs::write(dir.path().join("r1.hea"), h1).unwrap();
    std::fs::write(dir.path().join("r1.dat"), [0u8; 20]).unwrap();
    
    // Record 2
    let h2 = "r2 1 100 10\nr2.dat 16 200 0 0 0 0 0 s1";
    std::fs::write(dir.path().join("r2.hea"), h2).unwrap();
    std::fs::write(dir.path().join("r2.dat"), [0u8; 20]).unwrap();
    
    // Open directory
    let iterator = open(dir.path()).unwrap();
    let records: Vec<_> = iterator.collect::<Result<Vec<_>, _>>().unwrap();
    
    assert_eq!(records.len(), 2);
    let names: Vec<_> = records.iter().map(|r| r.header.metadata.name.clone()).collect();
    assert!(names.contains(&"r1".to_string()));
    assert!(names.contains(&"r2".to_string()));
}

