# Supertonic3 OpenAI

An OpenAI-compatible Text-to-Speech (TTS) HTTP API server that wraps the [Supertonic](https://huggingface.co/Supertone/supertonic-3) TTS model.

## Features

- **OpenAI API Compatible** - Drop-in replacement for OpenAI's TTS API endpoints
- **Multilingual Support** - 5 languages: English, Korean, Spanish, Portuguese, and French
- **10 Voice Styles** - Pre-configured voices (M1-M5 male, F1-F5 female)
- **GPU Acceleration** - Optional CUDA support for slightly faster inference (not optimized for GPU)

## Quick Start

### Prerequisites

- Rust
- ONNX Runtime (automatically handled by the `ort` crate)
- For GPU support: CUDA 12.9+, cuDNN 9.x+ and compatible NVIDIA drivers

### Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd supertonic-openai
```

1. Download the model assets:
   The preferred way is to use the [Hugginface CLI](https://huggingface.co/docs/huggingface_hub/guides/cli)

```bash
hf download Supertone/supertonic-3 --local-dir ./assets
```

1. Build the project:

```bash
# CPU version
cargo build --release

# GPU version
cargo build --release --features cuda
```

1. Run the server:

```bash
# Default settings (listens on 0.0.0.0:50051)
./target/release/supertonic-openai

# Custom settings
./target/release/supertonic-openai --gpu
```

## Docker Deployment

### CPU Version

```bash
# Build
docker build -t supertonic-openai .

# Run
docker run --rm -p 50051:50051 supertonic-openai
```

### GPU Version (CUDA)

```bash
# Build
docker build -f Dockerfile.cuda -t supertonic-openai:cuda .

# Run (requires NVIDIA Container Toolkit)
docker run --rm --gpus all -p 50051:50051 supertonic-openai:cuda
```

## Usage

### Command-Line Options

```
Options:
  -a, --assets-path <PATH>    Path to ONNX models [default: ./assets]
  -l, --listen <IP:PORT>      Server address [default: 0.0.0.0:50051]
      --gpu                   Enable GPU acceleration [default: false]
  -h, --help                  Print help
```

### API Endpoints

It implements the audio/speech and audio/voices [OpenAI endpoints](https://developers.openai.com/api/reference/resources/audio), as well as model listing endpoints for compatibility.

#### Generate Speech

**Request Body:**

```json
{
  "input": "Hello, this is a test of the Supertonic TTS system.",
  "model": "supertonic",
  "voice": "M1",
  "speed": 1.3,
  "total_step": 10,
  "silence_duration": 0.0,
  "language": "en" 
}
```

**Parameters:**

- `input` (required): Text to synthesize (string)
- `model` (required): Model name, must be "supertonic" (string)
- `voice` (optional): Voice style - M1-M5 (male) or F1-F5 (female), defaults to "M1" (string)
- `speed` (optional): Speech rate multiplier, defaults to 1.3 (float)
- `total_step` (optional): Denoising steps for quality (1-50), defaults to 10 (integer)
- `silence_duration` (optional): Pause duration between sentences in seconds, defaults to 0.0 (float)
- language (optional): Instruct which language to generate. English ("en") by default, otherwise you have to set this.

**Response:**

- Content-Type: `audio/wav`
- Body: WAV audio file (16-bit PCM, 24kHz sample rate)

**Example:**

```bash
curl -X POST http://localhost:50051/v1/audio/speech \
  -H "Content-Type: application/json" \
  -d '{
    "input": "Hello world!",
    "model": "supertonic",
    "voice": "F2",
    "speed": 1.0,
    "language": "en"
  }' \
  --output speech.wav
```
