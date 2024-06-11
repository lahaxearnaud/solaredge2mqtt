FROM debian:trixie-slim
RUN apt-get update && apt-get install -y curl libpq-dev
COPY target/release/rust-playground /rust-playground
CMD ["/rust-playground"]