FROM rust:buster as builder

WORKDIR /usr/src/skrillax
COPY . .

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo install --locked --path ./silkroad-agent
RUN cargo install --locked --path ./silkroad-gateway

FROM debian:buster

COPY --from=builder /usr/local/cargo/bin/silkroad-agent /usr/local/bin/silkroad-agent
COPY --from=builder /usr/local/cargo/bin/silkroad-gateway /usr/local/bin/silkroad-gateway

WORKDIR /opt/skrillax

COPY configs /opt/skrillax/configs/
COPY ./scripts/docker-run.sh /opt/skrillax/run.sh

CMD ["/opt/skrillax/run.sh"]