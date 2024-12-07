#!/bin/sh -eux

################################################################################
## This script shall conform to the POSIX.1 standard, a.k.a. IEEE Std 1003.1. ##
## When utilities defined in the standard are to be invoked, they shall only  ##
## be invoked utilizing functions defined by the standard, excluding any and  ##
## all extensions to the standard functions, e.g. GNU extensions.             ##
##                                                                            ##
## Version of the POSIX.1 standard used: POSIX.1-2008                         ##
## https://pubs.opengroup.org/onlinepubs/9699919799.2008edition/              ##
##                                                                            ##
## Used version of the standard should not be moved forward unless necessary  ##
## in order to keep the script as portable as possible between different      ##
## environments.                                                              ##
##                                                                            ##
## Used version of the standard should be moved backwards if possible in      ##
## order to keep the script as portable as possible between different         ##
## environments.                                                              ##
################################################################################

set -eux

readonly CHECK_DEPENDENCIES_UPDATED

RUSTFLAGS="${RUSTFLAGS:+"${RUSTFLAGS:?} "}-C link-arg=-s"
readonly RUSTFLAGS
export RUSTFLAGS

error_report_dir="/artifacts/"

error() {
  set -eu

  case "${#:?}" in
    ("1")
      "tee" \
        "/${error_report_dir:?}/error-report.txt" \
        >&2 \
        <<EOF
${1:?}
EOF
      ;;
    (*)
      "error" "\"error\" function requires exactly one argument! Number of \
passed arguments: ${#:?}."
      ;;
  esac

  exit "1"
}

is_privileged() (
  user_id="$("id" --user)"
  readonly user_id
  real_user_id="$(
    "id" \
      --real \
      --user
  )"
  readonly real_user_id
  
  for id in \
    "${user_id:?}" \
    "${real_user_id:?}"
  do
    case "${id:?}" in
      ("0")
        exit "0"
        ;;
    esac
  done

  exit "1"
)

list_contents() (
  case "${#:?}" in
    ("1")
      directory="${1:?}"
      readonly directory

      shift
      ;;
    (*)
      "error" "\"list_contents\" expects exactly one argument, the directory \
to list!"
      ;;
  esac

  cd "${directory:?}"

  "find" \
    "." \
    -path "./?*" \
    -a \
    "(" \
    "!" \
    -path "./?*/?**" \
    ")"
)

exec_for_dir_entries() (
  directory="${1:?}"
  command="${2:?}"
  error="${3:?}"
  shift 3

  files="$("list_contents" "${directory:?}")"
  readonly files

  case "${files?}" in
    ("") ;;
    (*)
      while read -r file
      do
        if ! (
          "${command:?}" \
            "${directory:?}" \
            "${file:?}" \
            "${@}"
        )
        then
          "${error:?}" \
            "${directory:?}" \
            "${file:?}"
        fi
      done <<EOF
${files:?}
EOF
      ;;
  esac
)

clean_dir_entry() {
  directory="${1:?}"
  file="${2:?}"
  shift 2

  "rm" \
    -f \
    -R \
    "${directory:?}/${file:?}"
}

clean_dir_entry_error() {
  directory="${1:?}"
  file="${2:?}"
  shift 2

  "error" "Failed to clean directory \"${directory:?}\"! Failed to remove \
\"${file:?}\"!"
}

clean_dir() (
  directory="${1:?}"
  shift

  "exec_for_dir_entries" \
    "${directory:?}" \
    "clean_dir_entry" \
    "clean_dir_entry_error"
)

run_unprivileged() {
  RUN_UNPRIVILEGED="1" \
    "setpriv" \
    --reuid="1000" \
    --regid="1000" \
    --clear-groups \
    --inh-caps="-all" \
    --no-new-privs \
    "${0:?}" \
    "${@:?}"
}

move_dir_contents_entry() {
  directory="${1:?}"
  file="${2:?}"
  target_directory="${3:?}"
  shift 3

  "mv" \
    "${directory:?}/${file:?}" \
    "${target_directory:?}"
}

move_dir_contents_entry_error() {
  directory="${1:?}"
  file="${2:?}"
  target_directory="${3:?}"
  shift 3

  "error" "Failed to move \"${file:?}\" from \"${directory:?}\" to \
\"${target_directory:?}\"!"
}

move_dir_contents() (
  directory="${1:?}"
  target_directory="${2:?}"
  shift 2

  "exec_for_dir_entries" \
    "${directory:?}" \
    "move_dir_contents_entry" \
    "move_dir_contents_entry_error" \
    "${target_directory:?}"
)

copy_dir_contents_entry() {
  directory="${1:?}"
  file="${2:?}"
  target_directory="${3:?}"
  shift 3

  "cp" \
    "${directory:?}/${file:?}" \
    "${target_directory:?}"
}

copy_dir_contents_entry_error() {
  directory="${1:?}"
  file="${2:?}"
  target_directory="${3:?}"
  shift 3

  "error" "Failed to copy \"${file:?}\" from \"${directory:?}\" to \
\"${target_directory:?}\"!"
}

copy_dir_contents() (
  directory="${1:?}"
  target_directory="${2:?}"
  shift 2

  "exec_for_dir_entries" \
    "${directory:?}" \
    "copy_dir_contents_entry" \
    "copy_dir_contents_entry_error" \
    "${target_directory:?}"
)

rerun_as_unprivileged() {
  if "is_privileged"
  then
    readonly error_report_dir

    build_configuration="/build-configuration/"
    readonly build_configuration

    protocol_json="${build_configuration:?}/protocol.json"
    readonly protocol_json

    topology_json="${build_configuration:?}/topology.json"
    readonly topology_json

    "clean_dir" "/artifacts/"

    for directory in \
      "target" \
      "rootless-artifacts"
    do
      if ! "mkdir" \
        -m "0755" \
        "/${directory:?}"
      then
        "error" "Failed to create \"/${directory:?}\" directory!"
      fi

      "chown" \
        "1000:1000" \
        "/${directory:?}"
    done

    if test -r "${protocol_json:?}" -o -r "${topology_json:?}"
    then
      protocol="$("cat" "${protocol_json:?}")"
      readonly protocol
      : "${protocol:?}"

      topology="$("cat" "${topology_json:?}")"
      readonly topology
      : "${topology:?}"

      "run_unprivileged" \
        "${@:?}" \
        "${protocol:?}" \
        "${topology:?}"
    else
      "run_unprivileged" "${@:?}"
    fi

    "rm" \
      -R \
      "/target"

    "chown" \
      -R \
      "0:0" \
      "/rootless-artifacts"

    "chmod" \
      -R \
      "0644" \
      "/rootless-artifacts"

    "move_dir_contents" \
      "/rootless-artifacts/" \
      "/artifacts/"

    "rmdir" "/rootless-artifacts"

    "copy_dir_contents" \
      "/labels/" \
      "/artifacts/"

    exit
  else
    : "${RUN_UNPRIVILEGED:?}"
    unset RUN_UNPRIVILEGED

    error_report_dir="/rootless-artifacts/"
    readonly error_report_dir
  fi
}

"rerun_as_unprivileged" "${@:?}"

check_groups() {
  group_ids="$("id" --groups)"
  readonly group_ids
  real_group_ids="$("id" --real --groups)"
  readonly real_group_ids

  for ids in "${group_ids:?}" "${real_group_ids:?}"
  do
    case "${ids:?}" in
      ("0"|"0"[![:digit:]]*|*[![:digit:]]"0"[![:digit:]]*|*[![:digit:]]"0")
      #("0"|"0"[[:digit:]]*|*[[:digit:]]"0"[[:digit:]]*|*[[:digit:]]"0")
        "error" "TODO"
        ;;
    esac
  done
}

"check_groups"

case "${CHECK_DEPENDENCIES_UPDATED:?}" in
  ("false") ;;
  (*)
    if ! "cargo" \
      "update" \
      --locked
    then
      "error" "Dependencies are out of date!"
    fi
    ;;
esac

if RELEASE_VERSION="$("cat" "/labels/release-version.txt")"
then
  readonly RELEASE_VERSION
  export RELEASE_VERSION
  : "${RELEASE_VERSION:?"Release version cannot be null!"}"
else
  "error" "Failed to read release version!"
fi

if cosmwasm_capabilities="$("cat" "/configuration/cosmwasm_capabilities")"
then
  readonly cosmwasm_capabilities
else
  "error" "Failed to read available CosmWasm capabilities!"
fi

build_profile="${1:?"Passing build profile as first parameter is required!"}"
readonly build_profile
shift

build_profiles_directory="/configuration/build-profiles/"
readonly build_profiles_directory

if mapped_build_profile="$(
  "cat" "${build_profiles_directory:?}/${build_profile:?}"
)"
then
  readonly mapped_build_profile

  : "${mapped_build_profile:?"Mapped build profile cannot be null!"}"
else
  if build_profiles="$(
    "ls" \
      -1 \
      "${build_profiles_directory:?}"
  )"
  then
    build_profiles_pretty=""

    while read -r build_profiles_entry
    do
      build_profiles_pretty="${build_profiles_pretty:+"${build_profiles_pretty:?}
"}* ${build_profiles_entry?}"
    done <<EOF
${build_profiles}
EOF

    "error" "Failed to read build profile mapping!

Existing profiles:
${build_profiles_pretty?}"
  else
    "error" "Failed to read available build profiles!"
  fi
fi

if max_binary_size="$(
  "cat" "/configuration/${build_profile:?}-max-binary-size"
)"
then
  readonly max_binary_size
  : "${max_binary_size:?"Maximum binary size cannot be null!"}"
else
  "error" "Failed to read max binary size for build profile!"
fi

if ! working_directory="$("pwd")"
then
  "echo" \
    "Failed to retrieve current directory's path!" \
    >&2
fi
if working_directory="$("basename" "${working_directory:?}")"
then
  readonly working_directory
  : "${working_directory:?}"
else
  "echo" \
    "Failed to retrieve name of current directory!" \
    >&2
fi

case "${working_directory:?}" in
  ("platform")
    case "${#:?}" in
      ("0") ;;
      (*)
        "error" "Expected exactly one argument denominating the build profile!"
        ;;
    esac

    dex_type=""
    readonly dex_type
    ;;
  ("protocol")
    protocol="${1:?}"
    topology="${2:?}"

    shift 2

    if protocol="$(
      "jq" \
        -c \
        "." \
        <<EOF
${protocol:?}
EOF
    )"
    then
      readonly protocol
      : "${protocol:?"Protocol JSON cannot be empty!"}"
    else
      "error" "Failed to parse protocol describing JSON!"
    fi

    if dex_type="$(
      "jq" \
        --exit-status \
        --raw-output \
        --argjson "protocol" "${protocol:?}" \
        ".networks[\$protocol.dex_network].dexes[\$protocol.dex].type | \
select(. != null)" \
        <<EOF
${topology:?}
EOF
    )"
    then
      unset topology

      readonly dex_type
      : "${dex_type:?"DEX type cannot be null!"}"
    else
      "error" "Failed to get DEX type from topology describing JSON file!"
    fi
    ;;
  (*)
    "error" "Current directory corresponds to an unknown workspace!"
    ;;
esac
readonly dex_type
: "${dex_type?}"

if ! contracts_count="$(
  "cat" "/configuration/${working_directory:?}-contracts-count"
)"
then
  "error" "Failed to read expected contracts count configuration for workspace!"
fi
readonly contracts_count
: "${contracts_count:?"Contracts count cannot be null!"}"

for directory in \
  "artifacts" \
  "target" \
  "rootless-artifacts"
do
  files="$(
    cd "/${directory:?}" && \
      "find" \
        "." \
        -path "?*/?**"
  )"

  case "${files?}" in
    ("") ;;
    (*)
      "error" "The \"${directory:?}\" directory is not empty!"
      ;;
  esac
done

CURRENCIES_BUILD_REPORT="/rootless-artifacts/currencies.build.log"
readonly CURRENCIES_BUILD_REPORT
export CURRENCIES_BUILD_REPORT

for tag in \
  "@agnostic" \
  "${dex_type:+"dex-${dex_type:?}"}"
do
  case "${tag?}" in
    ("") ;;
    (*)
      if ! "cargo" \
        -- \
        "each" \
        --tag "build" \
        --tag "${tag:?}" \
        "run" \
        --exact \
        -- \
        "build" \
        --profile "${mapped_build_profile:?}" \
        --lib \
        --frozen \
        --target "wasm32-unknown-unknown" \
        --target-dir "${CARGO_TARGET_DIR:?}"
      then
        "error" "Failed to build contracts in workspace tagged with \"${tag}\"!"
      fi
  esac
done

cargo_output_directory="${CARGO_TARGET_DIR:?}/wasm32-unknown-unknown/\
${mapped_build_profile:?}/"
readonly cargo_output_directory

if files="$(
  cd "${cargo_output_directory:?}" && \
    "find" \
      "." \
      "(" \
      "!" \
      "(" \
      -path "./?*/*" \
      -o \
      -path "./?*/*" \
      ")" \
      ")" \
      -type "f" \
      -name "*.wasm" \
      -print
)"
then
  if files="$(
    "sort" \
      <<EOF
${files:?}
EOF
  )"
  then
    readonly files
  else
    "error" "Failed to sort output directory files' paths via \"sort\"!"
  fi
else
  "error" "Failed to collect output directory files' paths!"
fi

if file_count="$(
  "wc" \
    -l \
    <<EOF
${files:?}
EOF
)"
then
  readonly file_count
else
  "error" "Failed to retrieve the output directories' binaries count via \"wc\"\
!"
fi

case "${file_count:?}" in
  ("0")
    "error" "Build produced no output! Expected ${contracts_count:?} contracts!"
    ;;
  ("${contracts_count:?}") ;;
  (*) "error" "Expected ${contracts_count:?} contracts, got ${file_count:?}!"
esac

while read -r wasm_path
do
  if ! wasm_name="$("basename" "${wasm_path:?}")"
  then
    "error" "Failed to extract basename from artifact file's path!"
  fi

  "echo" "Optimizing: ${wasm_name:?}"

  if "wasm-opt" \
    -Os \
    --inlining-optimizing \
    --signext-lowering \
    -o "/rootless-artifacts/${wasm_name:?}" \
    "${cargo_output_directory}/${wasm_path:?}"
  then
    "echo" "Finished optimizing: ${wasm_name:?}"
  else
    "error" "Failed to run \"wasm-opt\" on \"${wasm_name:?}\"!"
  fi
done \
  <<EOF
${files:?}
EOF

if large_files="$(
  cd "/rootless-artifacts/" && \
    "find" \
      "." \
      -type "f" \
      -name "*.wasm" \
      -size "+${max_binary_size:?}" \
      -printf "%f - %s bytes\n"
)"
then
  readonly large_files
else
  "error" "Failed to retrieve list of artifacts that are above allowed size!"
fi

case "${large_files?}" in
  ("") ;;
  (*) "error" "### These files are larger than the allowed limit, \
${max_binary_size:?}:
${large_files:?}"
esac

while read -r wasm_path
do
  (
    cd "/rootless-artifacts/" && \
      "cosmwasm-check" \
        --available-capabilities "${cosmwasm_capabilities?}" \
        "./${wasm_path:?}"
  )
done \
  <<EOF
${files:?}
EOF

case "${dex_type?}" in
  ("")
    build_output_packages=""
    ;;
  (*)
    build_output_packages="currencies"
    ;;
esac
readonly build_output_packages

case "${build_output_packages?}" in
  ("") ;;
  (*)
    outputs_directory="/rootless-artifacts/outputs"
    readonly outputs_directory

    "mkdir" "/${outputs_directory:?}"

    while read -r build_output_package
    do
      if build_output="$(
        cd "${cargo_output_directory:?}" && \
          "find" \
            "." \
            -type "d" \
            -path "./build/${build_output_package:?}-????????????????/out"
      )"
      then
        case "${build_output?}" in
          ("")
            "error" "Retrieved list of build script output directories doesn't \
    contain \"${build_output_package:?}-<FINGERPRINT>\" directories!"
            ;;
          (*)
            if read -r build_output \
              <<EOF
${build_output:?}
EOF
            then
              "mkdir" "/${outputs_directory:?}/${build_output_package:?}/"

              "cp" \
                "${cargo_output_directory:?}/${build_output:?}/"* \
                "/${outputs_directory:?}/${build_output_package:?}/"
            else
              "error" "Failed to retrieve first line of build script output \
    directories!"
            fi
        esac
      else
        "error" "Failed to list build script binaries' output directory!"
      fi
    done \
      <<EOF
${build_output_packages?}
EOF
    ;;
esac

if ! checksum="$(
  cd "/rootless-artifacts/" && \
    "sha256sum" \
      -- \
      "./"*".wasm"
)"
then
  "error" "Failed to calculate artifact checksums!"
fi
readonly checksum

if ! "tee" \
  "/rootless-artifacts/checksums.txt" \
  <<EOF

Checksums:
${checksum:?}
EOF
then
  "error" "Failed to write checksums to artifacts directory!"
fi
