mod opus;
mod resample;
mod wav;

pub enum ResponseFormat {
    Wav,
    OpusOgg,
}

impl TryFrom<String> for ResponseFormat {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "wav" => Ok(ResponseFormat::Wav),
            "opus" => Ok(ResponseFormat::OpusOgg),
            _ => Err(anyhow::anyhow!(
                "Unsupported response format: {}. Supported formats are wav and opus.",
                value
            )),
        }
    }
}

pub fn create_response(
    data: &[f32],
    sample_rate: u32,
    format: &ResponseFormat,
) -> anyhow::Result<Vec<u8>> {
    Ok(match format {
        ResponseFormat::Wav => wav::create_wav(data, sample_rate as i32)?,
        ResponseFormat::OpusOgg => opus::create_opus_ogg(data, sample_rate)?,
    })
}
