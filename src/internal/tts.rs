use crate::third_party::{TextToSpeech, load_text_to_speech};

pub fn load_model(assets_path: String, use_gpu: bool) -> anyhow::Result<TextToSpeech> {
    load_text_to_speech(&format!("{assets_path}/onnx"), use_gpu)
}
