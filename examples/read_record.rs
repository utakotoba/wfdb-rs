use std::env;
use wfdb::Record;

fn main() -> wfdb::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <record_path>", args[0]);
        std::process::exit(1);
    }

    let record_path = &args[1];

    let start_time = std::time::Instant::now();
    let record = Record::open(record_path)?;
    let duration = start_time.elapsed();

    println!("Loaded record in {:.2?} us", duration.as_micros());

    println!("Record: {}", record.metadata().name());
    println!(
        "Sampling frequency: {} Hz",
        record.metadata().sampling_frequency()
    );

    if let Some(num_samples) = record.metadata().num_samples {
        println!("Total samples: {num_samples}");
    }

    if record.is_multi_segment() {
        println!(
            "Multi-segment record with {} segments",
            record.segment_count()
        );
        if let Some(segments) = record.segment_info() {
            for (i, seg) in segments.iter().enumerate() {
                println!(
                    "  Segment {}: {} ({} samples)",
                    i, seg.record_name, seg.num_samples
                );
            }
        }
    } else {
        println!(
            "Single-segment record with {} signals",
            record.signal_count()
        );

        if let Some(signals) = record.signal_info() {
            for (i, sig) in signals.iter().enumerate() {
                println!(
                    "\nSignal {}: {}",
                    i,
                    sig.description.as_deref().unwrap_or("(no description)")
                );
                println!("  File: {}", sig.file_name);
                println!("  Format: {:?}", sig.format);
                println!(
                    "  Gain: {} ADC units per {}",
                    sig.adc_gain.unwrap_or(200.0),
                    sig.units.as_deref().unwrap_or("mV")
                );

                let mut reader = record.signal_reader(i)?;

                let adc_samples = reader.read_samples(10)?;
                println!(
                    "  First {} ADC samples: {:?}",
                    adc_samples.len(),
                    adc_samples
                );

                let mut reader = record.signal_reader(i)?;
                let physical_samples = reader.read_physical(10)?;
                println!(
                    "  First {} physical samples: {:?}",
                    physical_samples.len(),
                    physical_samples
                );
            }
        }
    }

    let total_duration = start_time.elapsed();
    println!("Total time: {:.2?} us", total_duration.as_micros());

    Ok(())
}
