# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine as builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen clang

RUN npm install -g sass pnpm

RUN curl --proto '=https' --tlsv1.3 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rust-src

WORKDIR /work
COPY . .

RUN pnpm install --frozen-lockfile
RUN pnpm build:core-js
RUN cargo leptos build --release -vv

FROM gcr.io/distroless/static-debian12:nonroot as runner

WORKDIR /app

COPY --from=builder --chown=65532:65532 /work/target/release/emerald-blog-v3 /app/
COPY --from=builder --chown=65532:65532 /work/target/site /app/site
COPY --from=builder --chown=65532:65532 /work/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 8080

USER 65532:65532
CMD ["/app/emerald-blog-v3"]
