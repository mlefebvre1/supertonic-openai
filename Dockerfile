FROM rust:slim-trixie as builder

RUN apt-get update && apt-get install -y build-essential \
                                         libssl-dev \ 
                                         pkg-config curl \ 
                                         python3 \
                                         python3-pip \
                                         python3-venv

WORKDIR /app

# Download ONNX stuff on Hugginface
RUN curl -LsSf https://hf.co/cli/install.sh | bash

RUN /root/.local/bin/hf download Supertone/supertonic-2 --local-dir ./assets

COPY . .
RUN cargo build --release

FROM rust:slim-trixie as runner

WORKDIR /app

COPY --from=builder /app/target/release/supertonic2-openai /app/supertonic2-openai
COPY --from=builder /app/assets /app/assets

ENTRYPOINT ["/app/supertonic2-openai"]

