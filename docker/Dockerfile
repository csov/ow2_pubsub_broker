# Very first stage builds the service
FROM ubuntu:latest AS builder

# The git key to use
ARG ssh_prv_key

RUN apt-get update\
 && apt-get install -y ca-certificates git curl clang cmake protobuf-compiler

# Authorize SSH Host
RUN mkdir -p /root/.ssh && chmod 0700 /root/.ssh && ssh-keyscan github.com > /root/.ssh/known_hosts

# Add the git key
RUN echo "$ssh_prv_key" > /root/.ssh/id_rsa && chmod 600 /root/.ssh/id_rsa
RUN echo "    IdentityFile /root/.ssh/id_rsa" >> /etc/ssh/ssh_config

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable && rustup update

# Takes few minutes to copy and build
COPY . /workspace
RUN cd /workspace && cargo build --release

# Following stage prepares the image to run the service
FROM ubuntu:latest AS runner

# The netcat is needed for healthchecks
RUN apt-get update && apt-get install -y ca-certificates netcat && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates

WORKDIR /run

# Messages gRPC service
FROM runner AS ow2_pubsub_broker
COPY --from=builder /workspace/target/release/ow2_pubsub_broker /run/ow2_pubsub_broker
CMD ["/run/ow2_pubsub_broker"]
