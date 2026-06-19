FROM rust:slim-trixie AS builder

RUN apt-get update && apt-get install -y build-essential \
                                         libssl-dev \
                                         pkg-config curl \
                                         libopus-dev \
                                         python3 \
                                         python3-pip \
                                         python3-venv

WORKDIR /app

# Download ONNX stuff on Hugginface
RUN curl -LsSf https://hf.co/cli/install.sh | bash

RUN /root/.local/bin/hf download Supertone/supertonic-3 --local-dir ./assets

# Copy sources and build
COPY . .
RUN cargo build --release


FROM rust:slim-trixie AS runner

RUN apt-get update && apt-get install -y libopus0

WORKDIR /app

# The actual app
COPY --from=builder /app/target/release/supertonic-openai /app/supertonic-openai

# ONNX models and configs
COPY --from=builder /app/assets /app/assets

ENTRYPOINT ["/app/supertonic-openai"]

