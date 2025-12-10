use std::env;
use wfdb::Record;

fn main() -> wfdb::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <record_path>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} ./references/wfdb-master/data/100s", args[0]);
        std::process::exit(1);
    }

    let record_path = &args[1];
    let record = Record::open(record_path)?;

    println!("=== Record Information ===");
    println!("Record: {}", record.metadata().name());
    println!(
        "Sampling frequency: {} Hz",
        record.metadata().sampling_frequency()
    );

    if let Some(num_samples) = record.metadata().num_samples {
        println!("Total samples: {num_samples}");
    }

    if record.is_multi_segment() {
        demonstrate_multi_segment_seeking(&record)?;
    } else {
        demonstrate_single_segment_seeking(&record)?;
    }

    Ok(())
}

#[allow(clippy::unwrap_used)]
fn demonstrate_single_segment_seeking(record: &Record) -> wfdb::Result<()> {
    println!("\n=== Single-Segment Record ===");
    println!("Signals: {}", record.signal_count());

    if record.signal_count() == 0 {
        println!("No signals to read");
        return Ok(());
    }

    let signal_idx = 0;
    let signal_info = &record.signal_info().unwrap()[signal_idx];
    println!(
        "\nDemonstrating with Signal {}: {}",
        signal_idx,
        signal_info
            .description
            .as_deref()
            .unwrap_or("(no description)")
    );

    println!("\n--- SignalReader: Sample-based Seeking ---");
    let mut reader = record.signal_reader(signal_idx)?;

    println!("Reading first 5 samples from start...");
    let samples = reader.read_samples(5)?;
    println!("  Samples 0-4: {samples:?}");
    println!("  Current position: {}", reader.position());

    println!("\nSeeking to sample 100...");
    match reader.seek_to_sample(100) {
        Ok(pos) => {
            println!("  Seek to position: {pos}");
            let samples = reader.read_samples(5)?;
            println!("  Samples 100-104: {samples:?}");
            println!("  Current position: {}", reader.position());

            println!("\nSeeking backward to sample 50...");
            reader.seek_to_sample(50)?;
            let samples = reader.read_samples(3)?;
            println!("  Samples 50-52: {samples:?}");
            println!("  Current position: {}", reader.position());
        }
        Err(e) => {
            println!("  Seeking not supported for this format: {e}");
        }
    }

    println!("\n--- SignalReader: Time-based Seeking ---");
    let mut reader = record.signal_reader(signal_idx)?;

    println!("Seeking to 1.0 second...");
    match reader.seek_to_time(1.0) {
        Ok(pos) => {
            println!("  Seek to sample position: {pos}");
            println!(
                "  (1.0 sec × {} Hz = {} samples)",
                record.metadata().sampling_frequency(),
                pos
            );
            let samples = reader.read_samples(3)?;
            println!("  Samples at 1.0s: {samples:?}");
        }
        Err(e) => {
            println!("  Time-based seeking failed: {e}");
        }
    }

    println!("\n--- MultiSignalReader: Frame-based Seeking ---");
    let mut multi_reader = record.multi_signal_reader()?;

    println!("Reading first 3 frames...");
    let frames = multi_reader.read_frames(3)?;
    for (i, frame) in frames.iter().enumerate() {
        println!("  Frame {i}: {frame:?}");
    }
    println!("  Current position: frame {}", multi_reader.position());

    println!("\nSeeking to frame 50...");
    match multi_reader.seek_to_frame(50) {
        Ok(pos) => {
            println!("  Seek to frame: {pos}");
            let frames = multi_reader.read_frames(2)?;
            for (i, frame) in frames.iter().enumerate() {
                println!("  Frame {}: {:?}", 50 + i, frame);
            }
            println!("  Current position: frame {}", multi_reader.position());
        }
        Err(e) => {
            println!("  Frame seeking not supported: {e}");
        }
    }

    println!("\n--- Physical Units Conversion ---");
    let mut reader = record.signal_reader(signal_idx)?;

    println!(
        "Signal gain: {} ADC units per {}",
        reader.gain(),
        reader.units()
    );
    println!("Signal baseline: {} ADC units", reader.baseline());

    let adc_values = reader.read_samples(5)?;
    println!("\nFirst 5 ADC values: {adc_values:?}");

    reader.seek_to_sample(0)?;
    let physical_values = reader.read_physical(5)?;
    println!("First 5 physical values: {physical_values:?}");

    println!("\nConversion example:");
    for i in 0..adc_values.len().min(3) {
        let adc = adc_values[i];
        let physical = physical_values[i];
        println!("  ADC {} → {} {}", adc, physical, reader.units());
    }

    Ok(())
}

fn demonstrate_multi_segment_seeking(record: &Record) -> wfdb::Result<()> {
    println!("\n=== Multi-Segment Record ===");
    println!("Segments: {}", record.segment_count());

    if let Some(segments) = record.segment_info() {
        println!("\nSegment details:");
        for (i, seg) in segments.iter().enumerate() {
            println!(
                "  Segment {}: {} ({} samples)",
                i, seg.record_name, seg.num_samples
            );
        }
    }

    println!("\n--- SegmentReader: Cross-segment Seeking ---");
    let mut reader = record.segment_reader()?;

    println!("Reading first 5 frames...");
    let frames = reader.read_frames(5)?;
    for (i, frame) in frames.iter().enumerate() {
        println!("  Frame {i}: {frame:?}");
    }
    println!(
        "  Current position: sample {} (segment {})",
        reader.position(),
        reader.current_segment()
    );

    println!("\nSeeking to sample 100...");
    match reader.seek_to_sample(100) {
        Ok(pos) => {
            println!("  Seek to sample: {pos}");
            println!("  Now at segment: {}", reader.current_segment());

            let frames = reader.read_frames(3)?;
            for (i, frame) in frames.iter().enumerate() {
                println!("  Frame {}: {:?}", 100 + i, frame);
            }
            println!("  Current position: sample {}", reader.position());
        }
        Err(e) => {
            println!("  Seeking failed: {e}");
        }
    }

    println!("\nReading sequentially across segments...");
    reader.seek_to_sample(0)?;
    let mut frame_count = 0;
    while let Some(_frame) = reader.read_frame()? {
        frame_count += 1;
        if frame_count >= 10 {
            println!("  Read {frame_count} frames across segments...");
            break;
        }
    }
    println!(
        "  Final position: sample {} (segment {})",
        reader.position(),
        reader.current_segment()
    );

    Ok(())
}
