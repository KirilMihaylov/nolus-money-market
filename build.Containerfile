################################################################################
##                         START : EDIT  HERE : START                         ##
################################################################################

ARG rust_ver="1.83-slim"

ARG platform_contracts_count="3"

ARG protocol_contracts_count="7"

ARG test_network_build_profile="test_nets_release"

ARG test_network_max_binary_size="5M"

ARG production_network_build_profile="production_nets_release"

ARG production_network_max_binary_size="5M"

ARG cosmwasm_capabilities="cosmwasm_1_1,cosmwasm_1_2,iterator,neutron,staking,\
stargate"

################################################################################
##                           END : EDIT  HERE : END                           ##
################################################################################

ARG cargo_target_dir="/target/"

FROM docker.io/library/rust:${rust_ver:?} AS builder-base

ARG rust_ver

LABEL rust_ver="${rust_ver:?}"

VOLUME ["/artifacts/"]

ARG cargo_target_dir

ENV CARGO_TARGET_DIR="${cargo_target_dir:?}"

WORKDIR "/"

RUN "$("rustup" "which" "rustc")" \
    --version \
    >"/rust-version.txt"

RUN ["mkdir", "-m", "0555", "/build/"]

RUN ["mkdir", "-m", "0555", "/build-profiles/"]

RUN ["mkdir", "-m", "0555", "/configuration/"]

RUN ["mkdir", "-m", "0755", "/target/"]

RUN ["mkdir", "-m", "0755", "/temp-artifacts/"]

ENTRYPOINT ["sh", "-e", "/build/build.sh"]

RUN ["rustup", "target", "add", "wasm32-unknown-unknown"]

RUN --mount=type=cache,target="/var/cache/apt",sharing="locked" \
  --mount=type=cache,target="/var/lib/apt",sharing="locked" \
  ["apt", "update"]

RUN --mount=type=cache,target="/var/cache/apt",sharing="locked" \
  --mount=type=cache,target="/var/lib/apt",sharing="locked" \
  ["apt", "install", "--yes", "coreutils", "git", "jq", "sed", "tar", "wget"]

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
    >"/build-profiles/test-net"

ARG test_network_max_binary_size

RUN "printf" \
    "%s" \
    "${test_network_max_binary_size:?}" \
    >"/configuration/test-net-max-binary-size"

ARG production_network_build_profile

RUN "printf" \
    "%s" \
    "${production_network_build_profile:?}" \
    >"/build-profiles/production-net"

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

ARG binaryen_ver="version_117"

LABEL binaryen_ver="${binaryen_ver:?}"

RUN "echo" \
    "${binaryen_ver}" \
    >"/binaryen-version.txt"

RUN --mount=type=cache,id="${binaryen_ver:?}",target="/binaryen/",sharing="locked" \
  "[" "-f" "/binaryen/binaryen.tar.gz" "]" || \
    "wget" "-O" "/binaryen/binaryen.tar.gz" "https://github.com/WebAssembly/\
binaryen/releases/download/${binaryen_ver:?}/binaryen-${binaryen_ver:?}-x86_64-\
linux.tar.gz"

RUN --mount=type=cache,id="${binaryen_ver:?}",target="/binaryen/",sharing="locked" \
  "[" "-d" "/binaryen/binaryen" "]" || \
    ( \
      cd "/binaryen/" && \
        "tar" "-xf" "./binaryen.tar.gz" && \
        "mv" "./binaryen-${binaryen_ver:?}" "./binaryen" \
    )

RUN --mount=type=cache,id="${binaryen_ver:?}",target="/binaryen/",sharing="locked",readonly \
  ["cp", "/binaryen/binaryen/bin/wasm-opt", "/usr/bin/"]

RUN ["cargo", "install", "--jobs", "1", "--force", "cosmwasm-check"]

FROM builder-base AS builder

ARG check_dependencies_updated="true"

ENV CHECK_DEPENDENCIES_UPDATED="${check_dependencies_updated:?}"

LABEL check_container_dependencies="${check_dependencies_updated:?}"

RUN --mount=type=bind,source="./platform/",target="/platform/",readonly \
  --mount=type=bind,source="./protocol/",target="/protocol/",readonly \
  --mount=type=bind,source="./tools/",target="/tools/",readonly \
  check_and_fetch() ( \
    cd "${1:?}" && \
      case "${check_dependencies_updated:?}" in \
      ("false") ;; \
      ("true") \
        "cargo" "update" --locked \
        ;; \
      (*) \
        "echo" "Build argument \"check_dependencies_updated\" must be a boolean value!" && \
          exit "1" \
        ;; \
      esac && \
      "cargo" "fetch" --locked \
  ) && \
    "check_and_fetch" "/platform/" && \
    "check_and_fetch" "/protocol/" && \
    "check_and_fetch" "/tools/"

RUN --mount=type=bind,source="./",target="/code/",readonly \
  cd "/code/" && \
    "git" "config" --global "core.excludeFile" "/code/.dockerignore" && \
    tag="$("git" "describe" --tags --abbrev="0")" && \
    readonly tag && \
    tag_commit="$("git" "show-ref" "${tag:?}" --hash --abbrev)" && \
    readonly tag_commit && \
    described="$("git" "describe" --tags --dirty)" && \
    readonly described && \
    "git" "config" --global --unset "core.excludeFile" && \
    "printf" \
      "tag=%s / %s" \
      "${tag_commit:?}" \
      "${described:?}" \
      >"/release-version.txt"

ARG cargo_target_dir

RUN --mount=type=bind,source="./tools/",target="/tools/",readonly \
  --mount=type=tmpfs,target="${cargo_target_dir:?}" \
  [ \
    "cargo", \
      "install", \
      "--jobs", "1", \
      "--offline", \
      "--path", "/tools/cargo-each/" \
  ]

COPY --chmod="0555" "./scripts/build-and-optimize.sh" "/build/build.sh"

COPY --chmod="0555" "./.cargo/" "/.cargo/"

FROM builder AS platform-builder-base

COPY --chmod="0555" "./platform/" "/platform/"

FROM platform-builder-base AS platform-builder

WORKDIR "/platform/"

FROM platform-builder-base AS protocol-builder

VOLUME ["/build-configuration/"]

WORKDIR "/protocol/"

COPY --chmod="0555" "./protocol/" "/protocol/"
