use std::{io::Read, path::Path};

use hound::{SampleFormat, WavSpec, WavWriter};

pub fn create_wav(audio_data: &[f32], sample_rate: i32) -> anyhow::Result<Vec<u8>> {
    let mut tmp = tempfile::NamedTempFile::with_suffix(".wav")?;
    write_wav_file(tmp.path(), audio_data, sample_rate)?;

    let mut out = vec![];
    tmp.read_to_end(&mut out)?;
    Ok(out)
}

pub fn write_wav_file<P: AsRef<Path>>(
    filename: P,
    audio_data: &[f32],
    sample_rate: i32,
) -> anyhow::Result<()> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(filename, spec)?;

    for &sample in audio_data {
        let clamped = sample.clamp(-1.0, 1.0);
        let val = (clamped * 32767.0) as i16;
        writer.write_sample(val)?;
    }

    writer.finalize()?;
    Ok(())
}
