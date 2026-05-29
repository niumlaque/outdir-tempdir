FROM rust:1-trixie

WORKDIR /workspace
COPY . /workspace
RUN mkdir -p /tmp/outdir-tempdir-consumer-test /tmp/outdir-tempdir-cargo-target-tmp
WORKDIR /workspace/tests/fixtures/consumer-crate
ENV TMPDIR=/tmp/outdir-tempdir-consumer-test
ENV CARGO_TARGET_TMPDIR=/tmp/outdir-tempdir-cargo-target-tmp
ENV OUTDIR_TEMPDIR_CONSUMER_DOCKER_TEST=1
ENV OUTDIR_TEMPDIR_CONSUMER_CARGO_TARGET_TMPDIR=/tmp/outdir-tempdir-cargo-target-tmp

CMD ["cargo", "test"]
