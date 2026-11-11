# Construir desde la raíz del repositorio:
#   docker build -f rust/Dockerfile -t simulador-circuitos-malla-rust .
#
# Etapa de compilación
FROM rust:1.96-slim AS build
# plotters dibuja texto a través de fontconfig: necesita pkg-config y las cabeceras al compilar.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libfontconfig1-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY rust/Cargo.toml rust/Cargo.lock ./
# Cachea las dependencias antes de copiar el código real.
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY rust/src ./src
RUN touch src/main.rs && cargo build --release

# Etapa de ejecución
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libfontconfig1 fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 appuser

WORKDIR /app
COPY --from=build /src/target/release/simulador-circuitos-malla /usr/local/bin/simulador
COPY templates ./templates
COPY static ./static

ENV APP_ROOT=/app
ENV PORT=8000

USER appuser
EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl --fail --silent "http://127.0.0.1:8000/api/health" || exit 1

CMD ["simulador"]
