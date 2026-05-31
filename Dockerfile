FROM --platform=$BUILDPLATFORM public.ecr.aws/x1i4i6b9/rust-musl-cross-builder:nightly AS rust_builder

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
    apt-get update && \
    apt-get install -y cmake unzip curl ca-certificates gnupg binaryen clang

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key \
        | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg && \
    echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_22.x nodistro main" \
        > /etc/apt/sources.list.d/nodesource.list && \
    apt-get update && \
    apt-get install -y nodejs

RUN npm install -g sass pnpm@10.34.1

RUN curl --proto '=https' --tlsv1.3 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rust-src

WORKDIR /work
COPY . .

ARG BIN_FEATURES=ssr
ARG LIB_FEATURES=hydrate
ARG TARGETARCH

RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
RUN --mount=type=cache,target=/work/core-js/.parcel-cache \
    pnpm build:core-js
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/cargo-target \
    case ${TARGETARCH} in \
        arm64) \
            echo "Compiling for ARM64" && \
            TARGET_TRIPLE=aarch64-unknown-linux-musl ;; \
        amd64) \
            echo "Compiling for AMD64" && \
            TARGET_TRIPLE=x86_64-unknown-linux-musl ;; \
        *) \
            echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    rustup target add ${TARGET_TRIPLE} && \
    LEPTOS_BIN_TARGET_TRIPLE=${TARGET_TRIPLE} \
    CARGO_TARGET_DIR=/cargo-target/${TARGETARCH} cargo leptos build --release -vv \
    --bin-features="${BIN_FEATURES}" \
    --lib-features="${LIB_FEATURES}" \
    --bin-cargo-args="--locked" \
    --lib-cargo-args="--locked" && \
    cp /cargo-target/${TARGETARCH}/${TARGET_TRIPLE}/release/emerald-blog-v3 /usr/local/bin/emerald-blog-v3 && \
    cp -r /work/target/site /usr/local/share/emerald-blog-site

FROM gcr.io/distroless/static-debian12:nonroot AS runner

WORKDIR /app

COPY --from=rust_builder --chown=65532:65532 /usr/local/bin/emerald-blog-v3 /app/
COPY --from=rust_builder --chown=65532:65532 /usr/local/share/emerald-blog-site /app/site
COPY --from=rust_builder --chown=65532:65532 /work/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 8080

USER 65532:65532
CMD ["/app/emerald-blog-v3"]
