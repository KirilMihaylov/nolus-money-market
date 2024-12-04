################################################################################
##                         START : EDIT  HERE : START                         ##
################################################################################

### Rust version
ARG rust_ver="1.83"

### Debian version
ARG debian_ver="bookworm"

### WebAssembly/Binaryen tag to use
ARG binaryen_ver="version_119"

### "cosmwasm-check" version
ARG cosmwasm_check_ver

### Workspace settings
ARG platform_contracts_count="3"
ARG protocol_contracts_count="7"

### Test-net build settings
ARG test_network_build_profile="test_nets_release"
ARG test_network_max_binary_size="5M"

### Production-net build settings
ARG production_network_build_profile="production_nets_release"
ARG production_network_max_binary_size="5M"

### Artifacts check settings
ARG cosmwasm_capabilities="cosmwasm_1_1,cosmwasm_1_2,iterator,neutron,staking,\
stargate"

################################################################################
##                           END : EDIT  HERE : END                           ##
################################################################################

ARG check_dependencies_updated="true"

FROM docker.io/debian:${debian_ver}-slim AS debian

FROM debian AS debian-updated

RUN ["apt", "update"]

RUN ["apt", "upgrade", "--yes"]

FROM debian-updated AS configuration

RUN ["mkdir", "-m", "0555", "/configuration"]

RUN ["mkdir", "-m", "0555", "/configuration/build-profiles"]

ARG platform_contracts_count

RUN "printf" \
    "%d" \
    "${platform_contracts_count:?}" \
    >"/configuration/platform-contracts-count"

ARG protocol_contracts_count

RUN "printf" \
    "%d" \
    "${protocol_contracts_count:?}" \
    >"/configuration/protocol-contracts-count"

ARG test_network_build_profile

RUN "printf" \
    "%s" \
    "${test_network_build_profile:?}" \
    >"/configuration/build-profiles/test-net"

ARG test_network_max_binary_size

RUN "printf" \
    "%s" \
    "${test_network_max_binary_size:?}" \
    >"/configuration/test-net-max-binary-size"

ARG production_network_build_profile

RUN "printf" \
    "%s" \
    "${production_network_build_profile:?}" \
    >"/configuration/build-profiles/production-net"

ARG production_network_max_binary_size

RUN "printf" \
    "%s" \
    "${production_network_max_binary_size:?}" \
    >"/configuration/production-net-max-binary-size"

ARG cosmwasm_capabilities

RUN "printf" \
    "%s" \
    "${cosmwasm_capabilities:?}" \
    >"/configuration/cosmwasm_capabilities"

FROM debian-updated AS wasm-opt

RUN ["mkdir", "-m", "0555", "/labels"]

RUN ["apt", "install", "--yes", "coreutils", "tar", "wget"]

WORKDIR "/binaryen/"

ARG binaryen_ver

RUN "wget" \
    "-O" "./binaryen.tar.gz" \
    "https://github.com/WebAssembly/binaryen/releases/download/\
${binaryen_ver:?}/binaryen-${binaryen_ver:?}-x86_64-linux.tar.gz"

RUN ["tar", "-x", "-f", "./binaryen.tar.gz"]

RUN "printf" \
    "%s" \
    "${binaryen_ver:?}" \
    >"/labels/binaryen-version.txt"

RUN "mv" \
    "./binaryen-${binaryen_ver:?}/bin/wasm-opt" \
    "./"

ARG rust_ver

FROM docker.io/rust:${rust_ver:?}-slim AS rust

RUN ["mkdir", "-m", "0555", "/labels"]

RUN "chmod" "-R" "a-w" "${CARGO_HOME:?}"

RUN "chown" "-R" "0:0" "${CARGO_HOME:?}"

RUN "chmod" "-R" "a-w" "${RUSTUP_HOME:?}"

RUN "chown" "-R" "0:0" "${RUSTUP_HOME:?}"

ARG rust_ver

LABEL rust_ver="${rust_ver:?}"

RUN rustc_bin="$("rustup" "which" "rustc")" && \
      rust_version="$("${rustc_bin:?}" --version)" && \
      "printf" \
        "%s" \
        "${rust_version:?}" \
        >"/labels/rust-version.txt"

RUN ["apt", "update"]

RUN ["apt", "upgrade", "--yes"]

FROM rust AS rust-with-wasm32-target

RUN ["rustup", "target", "add", "wasm32-unknown-unknown"]

FROM rust AS cosmwasm-check

ARG cosmwasm_check_ver

RUN "cargo" \
    "install" \
    "--force" \
    "--jobs" "1" \
    "cosmwasm-check@${cosmwasm_check_ver:?}"

FROM rust-with-wasm32-target AS builder-base

ENV CARGO_TARGET_DIR="/target/"

WORKDIR "/"

RUN ["mkdir", "-m", "0700", "/output"]

VOLUME ["/output/artifacts/"]

RUN ["mkdir", "-m", "0111", "/build"]

ENTRYPOINT ["/build/build.sh"]

RUN ["apt", "install", "--yes", "coreutils", "jq", "procps", "util-linux"]

COPY \
  --chmod="0444" \
  --chown="0:0" \
  --from=configuration \
  "/configuration" \
  "/configuration"

RUN [ \
    "/usr/bin/find", \
    "/configuration", \
    "-type", "d", \
    "-exec", "/usr/bin/chmod", "0555", "{}", ";" \
  ]

COPY \
  --chmod="0111" \
  --chown="0:0" \
  --from=cosmwasm-check \
  "/usr/local/cargo/bin/cosmwasm-check" \
  "/usr/local/cargo/bin/"

COPY \
  --chmod="0111" \
  --chown="0:0" \
  --from=wasm-opt \
  "/binaryen/wasm-opt" \
  "/usr/bin/"

COPY \
  --chmod="0444" \
  --chown="0:0" \
  --from=wasm-opt \
  "/labels" \
  "/labels"

RUN [ \
    "/usr/bin/find", \
    "/labels", \
    "-type", "d", \
    "-exec", "/usr/bin/chmod", "0555", "{}", ";" \
  ]

FROM rust AS cargo-each

COPY \
  --chmod="0444" \
  --chown="0:0" \
  "./.cargo" \
  "/.cargo"

RUN [ \
    "/usr/bin/find", \
      "/.cargo", \
      "-type", "d", \
      "-exec", "/usr/bin/chmod", "0555", "{}", ";" \
  ]

RUN --mount=type=bind,source="./tools/",target="/tools/",readonly \
  [ \
    "cargo", \
      "fetch", \
      "--manifest-path", "/tools/cargo-each/Cargo.toml", \
      "--locked" \
  ]

RUN --mount=type=bind,source="./tools/",target="/tools/",readonly \
  --mount=type=tmpfs,target="/target/" \
  [ \
    "cargo", \
      "install", \
      "--force", \
      "--jobs", "1", \
      "--locked", \
      "--path", "/tools/cargo-each/", \
      "--target-dir", "/target/" \
  ]

FROM debian-updated AS release-version-label

RUN ["mkdir", "-m", "0555", "/labels"]

RUN ["apt", "install", "--yes", "coreutils", "git"]

RUN --mount=type=bind,source="./",target="/code/",readonly \
  cd "/code" && \
    tag="$("git" "describe" --tags)" && \
    readonly tag && \
    latest_tag="$("git" "describe" --tags --abbrev="0")" && \
    readonly latest_tag && \
    tag_commit="$(\
      "git" \
        "show-ref" \
        "${latest_tag:?}" \
        --abbrev \
        --hash \
        --tags\
    )" && \
    readonly tag_commit && \
    "printf" \
      "tag=%s / %s" \
      "${tag_commit:?}" \
      "${tag:?}" \
      >"/labels/release-version.txt"

FROM builder-base AS builder

COPY \
  --chmod="0111" \
  --chown="0:0" \
  --from=cargo-each \
  "/usr/local/cargo/bin/cargo-each" \
  "/usr/local/cargo/bin/"

COPY \
  --chmod="0444" \
  --chown="0:0" \
  --from=release-version-label \
  "/labels" \
  "/labels"

RUN [ \
    "/usr/bin/find", \
    "/labels", \
    "-type", "d", \
    "-exec", "/usr/bin/chmod", "0555", "{}", ";" \
  ]

COPY \
  --chmod="0555" \
  --chown="0:0" \
  "./scripts/build-and-optimize.sh" \
  "/build/build.sh"

COPY \
  --chmod="0444" \
  --chown="0:0" \
  "./platform" \
  "/platform"

RUN [ \
    "/usr/bin/find", \
    "/platform", \
    "-type", "d", \
    "-exec", "/usr/bin/chmod", "0555", "{}", ";" \
  ]

ARG check_dependencies_updated

ENV CHECK_DEPENDENCIES_UPDATED="${check_dependencies_updated:?}"

LABEL check_dependencies_updated="${check_dependencies_updated:?}"

RUN --mount=type=bind,source="./protocol/",target="/protocol/",readonly \
  --mount=type=bind,source="./tools/",target="/tools/",readonly \
  check_and_fetch() ( \
    cd "${1:?}" && \
      case "${check_dependencies_updated:?}" in \
        ("false") ;; \
        ("true") \
          "cargo" "update" --locked \
          ;; \
        (*) \
          "echo" "Build argument \"check_dependencies_updated\" must be a \
boolean value!" && \
            exit "1" \
          ;; \
      esac && \
      "cargo" "fetch" --locked \
  ) && \
    "check_and_fetch" "/platform/" && \
    "check_and_fetch" "/protocol/"

FROM builder AS platform-builder

WORKDIR "/platform/"

FROM builder AS protocol-builder

WORKDIR "/protocol/"

RUN ["mkdir", "-m", "0500", "/inputs"]

VOLUME ["/inputs/build-configuration/"]

COPY \
  --chmod="0444" \
  --chown="0:0" \
  "./protocol" \
  "/protocol"

RUN [ \
    "/usr/bin/find", \
    "/protocol", \
    "-type", "d", \
    "-exec", "/usr/bin/chmod", "0555", "{}", ";" \
  ]

COPY \
  --chmod="0444" \
  --chown="0:0" \
  "./tools" \
  "/tools"

RUN [ \
    "/usr/bin/find", \
    "/tools", \
    "-type", "d", \
    "-exec", "/usr/bin/chmod", "0555", "{}", ";" \
  ]
