FROM ubuntu:12.04

RUN pwd
RUN ls -al

# Install zip and curl so that we can download and unzip the source repo.
RUN apt-get update && \
    apt-get install -qqy --no-install-recommends \
    unzip \
    curl \
    ca-certificates \
    gcc \
    libc6-dev

# Install the nightly Rust toolchain via rustup.
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

# Download the app from master and unzip it.
RUN curl -o master.zip -L https://codeload.github.com/excaliburHisSheath/hangry-river-horse/zip/master
RUN unzip master.zip

# Try building the repo using cargo.
RUN cd hangry-river-horse-master && $HOME/.cargo/bin/cargo build

EXPOSE 80 6767 6768 6769

ENV ROCKET_ENV=production

CMD ["/hangry-river-horse-master/target/debug/hangry-hippos"]
