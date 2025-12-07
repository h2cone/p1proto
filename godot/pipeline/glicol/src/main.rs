use std::{env, fs, path::PathBuf, process::exit};

use glicol::Engine;
use hound::{SampleFormat, WavSpec, WavWriter};

const BLOCK: usize = 128;

struct Config {
    input: PathBuf,
    output: PathBuf,
    duration_secs: f32,
    sample_rate: u32,
}

fn usage() -> &'static str {
    "Usage: glicol-exporter <input.glicol> [-o output.wav] [--duration SECONDS] [--sr SAMPLE_RATE]"
}

fn parse_args() -> Result<Config, String> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Missing input .glicol file".to_string())?;

    let mut output: Option<PathBuf> = None;
    let mut duration_secs: f32 = 60.0;
    let mut sample_rate: u32 = 44_100;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = args.next().map(PathBuf::from);
                if output.is_none() {
                    return Err("Expected path after -o/--output".into());
                }
            }
            "--duration" | "-d" => {
                let Some(val) = args.next() else {
                    return Err("Expected seconds after --duration/-d".into());
                };
                duration_secs = val
                    .parse::<f32>()
                    .map_err(|_| "Duration must be a number (seconds)".to_string())?;
                if duration_secs <= 0.0 {
                    return Err("Duration must be > 0".into());
                }
            }
            "--sr" | "--sample-rate" => {
                let Some(val) = args.next() else {
                    return Err("Expected value after --sr/--sample-rate".into());
                };
                sample_rate = val
                    .parse::<u32>()
                    .map_err(|_| "Sample rate must be a positive integer".to_string())?;
                if sample_rate == 0 {
                    return Err("Sample rate must be > 0".into());
                }
            }
            other => return Err(format!("Unknown arg: {other}")),
        }
    }

    let output = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("wav");
        path
    });

    Ok(Config {
        input,
        output,
        duration_secs,
        sample_rate,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match parse_args() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{err}");
            eprintln!("{usage}", usage = usage());
            exit(1);
        }
    };

    let code = fs::read_to_string(&config.input)?;

    let mut engine = Engine::<BLOCK>::new();
    engine.set_sr(config.sample_rate as usize);
    // Some glicol versions return Result<(), _>, some return ().
    let _ = engine.update_with_code(&code);

    if let Some(parent) = config.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let spec = WavSpec {
        channels: 2,
        sample_rate: config.sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(&config.output, spec)?;

    let mut frames_left = (config.duration_secs * config.sample_rate as f32).ceil() as usize;
    while frames_left > 0 {
        let buffers = engine.next_block(vec![]);
        let frames_in_block = buffers.first().map(|b| b.len()).unwrap_or(0);
        if frames_in_block == 0 {
            break;
        }
        let take = frames_in_block.min(frames_left);

        for i in 0..take {
            let l = buffers.get(0).map(|b| b[i]).unwrap_or(0.0);
            let r = buffers.get(1).map(|b| b[i]).unwrap_or(l);
            let l = (l * 0.9).clamp(-1.0, 1.0);
            let r = (r * 0.9).clamp(-1.0, 1.0);
            writer.write_sample((l * i16::MAX as f32) as i16)?;
            writer.write_sample((r * i16::MAX as f32) as i16)?;
        }

        frames_left -= take;
    }

    writer.finalize()?;
    println!(
        "Wrote {} seconds to {}",
        config.duration_secs,
        config.output.display()
    );
    Ok(())
}
