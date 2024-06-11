FROM debian:trixie-slim
RUN apt-get update && apt-get install -y curl libpq-dev
COPY target/release/solaredge2mqtt /solaredge2mqtt
CMD ["/solaredge2mqtt"]
