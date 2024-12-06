################################################################################
##                         START : EDIT  HERE : START                         ##
################################################################################

ARG rust_ver="1.83-slim"

ARG nightly_rust_ver="nightly"

################################################################################
##                           END : EDIT  HERE : END                           ##
################################################################################

FROM docker.io/library/rust:${rust_ver:?} AS builder

USER 0:0

RUN ["mkdir", "-m", "01557", "/build/"]

ENV CARGO_TARGET_DIR="/build/target/"

RUN ["apt", "update"]

RUN ["apt", "upgrade", "--yes"]

RUN ["apt", "install", "--yes", "gcc", "libssl-dev", "pkg-config"]

FROM builder AS cargo-audit

RUN ["cargo", "install", "--jobs", "1", "--force", "cargo-audit"]

FROM builder AS cargo-each

RUN \
    --mount=type=bind,source="./tools/",target="/code/",readonly \
    --mount=type=tmpfs,target="/code/target/" \
    [ \
        "cargo", \
        "install", \
        "--jobs", "1", \
        "--force", \
        "--path", "/code/cargo-each/" \
    ]

FROM builder AS cargo-udeps

RUN ["cargo", "install", "--jobs", "1", "--force", "cargo-udeps"]

FROM builder

ENTRYPOINT ["/check.sh"]

RUN ["chmod", "-R", "01557", "/usr/local/cargo/"]

RUN ["chmod", "-R", "0555", "/usr/local/cargo/bin/"]

VOLUME ["/code/"]

RUN ["chmod", "0555", "/code/"]

RUN ["rustup", "component", "add", "clippy", "rustfmt"]

RUN ["rustup", "target", "add", "wasm32-unknown-unknown"]

COPY \
    --chmod="0555" \
    --chown=0:0 \
    --from=cargo-audit \
    "/usr/local/cargo/bin/" \
    "/usr/local/cargo/bin/"

COPY \
    --chmod="0555" \
    --chown=0:0 \
    --from=cargo-udeps \
    "/usr/local/cargo/bin/" \
    "/usr/local/cargo/bin/"

ARG nightly_rust_ver

RUN "rustup" "toolchain" "add" "${nightly_rust_ver}"

USER 1000:1000

COPY \
    --chmod="0555" \
    --chown=0:0 \
    --from=cargo-each \
    "/usr/local/cargo/bin/" \
    "/usr/local/cargo/bin/"

COPY \
    --chmod="0555" \
    --chown=0:0 \
    "./scripts/check/*.sh" \
    "/"
