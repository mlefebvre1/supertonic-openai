use std::{
    io::Cursor,
    time::{SystemTime, UNIX_EPOCH},
};

use ogg::{PacketWriteEndInfo, PacketWriter};

const BITRATE: i32 = 64_000;
const FRAME_SIZE_MS: usize = 20;

pub fn create_opus_ogg(pcm_data: &[f32], input_sample_rate: u32) -> anyhow::Result<Vec<u8>> {
    tracing::info!(
        "Starting Opus encoding. Sample rate: {} Hz, PCM data length: {} samples.",
        input_sample_rate,
        pcm_data.len()
    );

    const SAMPLE_RATE: u32 = 48_000;

    //TODO: resample from sample_rate to 48000 if needed, since Opus encoder expects 48kHz input.
    //For now we assume the input is always 48kHz.

    let mut encoder =
        opus::Encoder::new(SAMPLE_RATE, opus::Channels::Mono, opus::Application::Audio)?;
    encoder.set_bitrate(opus::Bitrate::Bits(BITRATE))?;

    tracing::info!(
        "Opus Encoder initialized with sample rate {} Hz and bitrate {} bps.",
        SAMPLE_RATE,
        BITRATE
    );

    let serial_no = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(1);

    let opus_header = create_opus_header(SAMPLE_RATE);
    let opus_tags = create_opus_tag();

    let mut ogg_buffer = Cursor::new(Vec::new());
    let mut packet_writer = PacketWriter::new(&mut ogg_buffer);

    packet_writer.write_packet(&opus_header, serial_no, PacketWriteEndInfo::EndPage, 0)?;
    packet_writer.write_packet(&opus_tags, serial_no, PacketWriteEndInfo::EndPage, 0)?;

    // Actual encoding
    let frame_size = (FRAME_SIZE_MS * SAMPLE_RATE as usize) / 1000;
    let granule_step = frame_size as u64 * 48000 / SAMPLE_RATE as u64;

    // Output buffer recommendation: 4000 bytes is generally enough for max Opus frame
    let mut encode_buffer = vec![0u8; 4000];
    let mut granule_pos = 0u64;
    for chunk in pcm_data.chunks(frame_size) {
        // Padding for last chunk
        let input_frame = if chunk.len() < frame_size {
            let mut padded = chunk.to_vec();
            padded.resize(frame_size, 0.0);
            std::borrow::Cow::Owned(padded)
        } else {
            std::borrow::Cow::Borrowed(chunk)
        };

        let encoded_len = encoder.encode_float(&input_frame, &mut encode_buffer)?;

        let is_last_chunk = chunk.len() < frame_size;

        let end_info = if is_last_chunk {
            PacketWriteEndInfo::EndStream
        } else {
            PacketWriteEndInfo::NormalPacket
        };
        let packet_data = encode_buffer[..encoded_len].to_vec();
        packet_writer.write_packet(packet_data, serial_no, end_info, granule_pos)?;
        granule_pos += granule_step;
    }

    drop(packet_writer);

    Ok(ogg_buffer.into_inner())
}

fn create_opus_header(sample_rate: u32) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(b"OpusHead");
    header.push(1); // version
    header.push(1); // channel count
    header.extend_from_slice(0u16.to_le_bytes().as_ref()); // pre-skip
    header.extend_from_slice(sample_rate.to_le_bytes().as_ref()); // sample rate
    header.extend_from_slice(0u16.to_le_bytes().as_ref()); // output gain
    header.push(0); // channel mapping family
    header
}

fn create_opus_tag() -> Vec<u8> {
    let mut tags = Vec::new();
    tags.extend_from_slice(b"OpusTags");
    let vendor = b"hal9k";
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes()); // vendor string length
    tags.extend_from_slice(vendor); // vendor string
    tags.extend_from_slice(0u32.to_le_bytes().as_ref()); // user comment list length (0 for no
    tags
}
