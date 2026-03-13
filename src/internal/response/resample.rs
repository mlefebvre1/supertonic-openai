use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};

pub fn resample(fs_in: u32, fs_out: u32, indata: &[f32]) -> anyhow::Result<Vec<f32>> {
    let mut resampler =
        Fft::<f32>::new(fs_in as usize, fs_out as usize, 1024, 1, 1, FixedSync::Both)?;

    let fs_ratio = fs_out as f32 / fs_in as f32;

    let mut outdata = vec![0.0f32; 2 * (indata.len() as f32 * fs_ratio) as usize];

    let mut indexing = Indexing {
        input_offset: 0,
        output_offset: 0,
        active_channels_mask: None,
        partial_len: None,
    };

    let nbr_input_frames = indata.len();
    let nbr_output_frames = outdata.len();
    let mut input_frames_next = resampler.input_frames_next();

    let input_adapter = InterleavedSlice::new(indata, 1, nbr_input_frames)?;
    let mut output_adapter = InterleavedSlice::new_mut(&mut outdata, 1, nbr_output_frames)?;
    let mut input_frames_left = nbr_input_frames;
    while input_frames_left >= input_frames_next {
        let (nbr_in, nbr_out) = resampler
            .process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))
            .unwrap();

        indexing.input_offset += nbr_in;
        indexing.output_offset += nbr_out;
        input_frames_left -= nbr_in;
        input_frames_next = resampler.input_frames_next();
    }
    indexing.partial_len = Some(input_frames_left);
    let (_nbr_in, _nbr_out) =
        resampler.process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))?;

    Ok(outdata)
}
