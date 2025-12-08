use std::{
    env, fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::exit,
};

use glicol::Engine;
use hound::{SampleFormat, WavSpec, WavWriter};
use mp3lame_encoder::{
    Bitrate, Builder as Mp3Builder, FlushGap, InterleavedPcm, Quality as Mp3Quality,
};
use vorbis_encoder::Encoder as VorbisEncoder;

const BLOCK: usize = 128;

struct Config {
    input: PathBuf,
    output: PathBuf,
    duration_secs: f32,
    sample_rate: u32,
}

fn usage() -> &'static str {
    "Usage: glicol-exporter <input.glicol> [-o output.(wav|ogg|mp3)] [--duration SECONDS] [--sr SAMPLE_RATE]"
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

#[derive(Copy, Clone)]
enum AudioFormat {
    Wav,
    Ogg,
    Mp3,
}

impl AudioFormat {
    fn from_path(path: &Path) -> Result<Self, String> {
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            return Err(
                "Output path must include an extension (.wav, .ogg/.oga, or .mp3)".to_string(),
            );
        };
        match ext.to_ascii_lowercase().as_str() {
            "wav" | "wave" => Ok(Self::Wav),
            "ogg" | "oga" => Ok(Self::Ogg),
            "mp3" => Ok(Self::Mp3),
            other => Err(format!(
                "Unsupported output extension: .{other}. Use wav, ogg, or mp3."
            )),
        }
    }
}

enum AudioSink {
    Wav(WavWriter<BufWriter<fs::File>>),
    Ogg {
        encoder: VorbisEncoder,
        file: BufWriter<fs::File>,
    },
    Mp3 {
        encoder: mp3lame_encoder::Encoder,
        file: BufWriter<fs::File>,
    },
}

impl AudioSink {
    fn new(
        path: &Path,
        format: AudioFormat,
        sample_rate: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match format {
            AudioFormat::Wav => {
                let spec = WavSpec {
                    channels: 2,
                    sample_rate,
                    bits_per_sample: 16,
                    sample_format: SampleFormat::Int,
                };
                let writer = WavWriter::create(path, spec)?;
                Ok(Self::Wav(writer))
            }
            AudioFormat::Ogg => {
                let encoder = VorbisEncoder::new(2, sample_rate as u64, 0.5)
                    .map_err(|code| format!("Failed to init Vorbis encoder (code {code})"))?;
                let file = BufWriter::new(fs::File::create(path)?);
                Ok(Self::Ogg { encoder, file })
            }
            AudioFormat::Mp3 => {
                let mut builder = Mp3Builder::new().ok_or("Failed to allocate MP3 encoder")?;
                builder
                    .set_num_channels(2)
                    .map_err(|e| format!("Failed to set MP3 channels: {e}"))?;
                builder
                    .set_sample_rate(sample_rate)
                    .map_err(|e| format!("Failed to set MP3 sample rate: {e}"))?;
                builder
                    .set_brate(Bitrate::Kbps192)
                    .map_err(|e| format!("Failed to set MP3 bitrate: {e}"))?;
                builder
                    .set_quality(Mp3Quality::Best)
                    .map_err(|e| format!("Failed to set MP3 quality: {e}"))?;
                let encoder = builder
                    .build()
                    .map_err(|e| format!("Failed to build MP3 encoder: {e}"))?;
                let file = BufWriter::new(fs::File::create(path)?);
                Ok(Self::Mp3 { encoder, file })
            }
        }
    }

    fn write_block(&mut self, samples: &Vec<i16>) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            AudioSink::Wav(writer) => {
                for sample in samples {
                    writer.write_sample(*sample)?;
                }
            }
            AudioSink::Ogg { encoder, file } => {
                let encoded = encoder
                    .encode(samples)
                    .map_err(|code| format!("Ogg Vorbis encode error (code {code})"))?;
                file.write_all(&encoded)?;
            }
            AudioSink::Mp3 { encoder, file } => {
                let sample_pairs = samples.len() / 2;
                let mut buffer = Vec::<u8>::with_capacity(
                    mp3lame_encoder::max_required_buffer_size(sample_pairs),
                );
                encoder
                    .encode_to_vec(InterleavedPcm(samples), &mut buffer)
                    .map_err(|e| format!("MP3 encode error: {e}"))?;
                file.write_all(&buffer)?;
            }
        }
        Ok(())
    }

    fn finalize(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            AudioSink::Wav(writer) => {
                writer.finalize()?;
            }
            AudioSink::Ogg {
                mut encoder,
                mut file,
            } => {
                let encoded = encoder
                    .flush()
                    .map_err(|code| format!("Ogg Vorbis flush error (code {code})"))?;
                file.write_all(&encoded)?;
                file.flush()?;
            }
            AudioSink::Mp3 {
                mut encoder,
                mut file,
            } => {
                let mut buffer = Vec::<u8>::with_capacity(7200);
                encoder
                    .flush_to_vec::<FlushGap>(&mut buffer)
                    .map_err(|e| format!("MP3 flush error: {e}"))?;
                file.write_all(&buffer)?;
                file.flush()?;
            }
        }
        Ok(())
    }
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

    let format = AudioFormat::from_path(&config.output)?;
    let mut sink = AudioSink::new(&config.output, format, config.sample_rate)?;

    let mut frames_left = (config.duration_secs * config.sample_rate as f32).ceil() as usize;
    let mut interleaved: Vec<i16> = Vec::with_capacity(BLOCK * 2);
    while frames_left > 0 {
        let buffers = engine.next_block(vec![]);
        let frames_in_block = buffers.first().map(|b| b.len()).unwrap_or(0);
        if frames_in_block == 0 {
            break;
        }
        let take = frames_in_block.min(frames_left);

        interleaved.clear();
        interleaved.reserve(take * 2);
        for i in 0..take {
            let l = buffers.get(0).map(|b| b[i]).unwrap_or(0.0);
            let r = buffers.get(1).map(|b| b[i]).unwrap_or(l);
            let l = (l * 0.9).clamp(-1.0, 1.0);
            let r = (r * 0.9).clamp(-1.0, 1.0);
            interleaved.push((l * i16::MAX as f32) as i16);
            interleaved.push((r * i16::MAX as f32) as i16);
        }

        sink.write_block(&interleaved)?;
        frames_left -= take;
    }

    sink.finalize()?;
    println!(
        "Wrote {duration} seconds to {output} ({format})",
        duration = config.duration_secs,
        output = config.output.display(),
        format = match format {
            AudioFormat::Wav => "wav",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Mp3 => "mp3",
        }
    );
    Ok(())
}
