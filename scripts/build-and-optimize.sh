#!/bin/sh -eu

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

################################################################################
## Utilities used that are not defined by the POSIX standard:                 ##
## * "setpriv" from "util-linux"                                              ##
## * "jq" from self-named package                                             ##
## * "cargo" from Rustup toolchain distribution                               ##
## * "cargo-each" from local sources                                          ##
## * "wasm-opt" from WebAssembly/Binaryen                                     ##
## * "cosmwasm-check" from self-named package from "crates.io"                ##
## * "sha256sum" from GNU "coreutils"                                         ##
################################################################################

set -eu

readonly CHECK_DEPENDENCIES_UPDATED

readonly CARGO_TARGET_DIR

RUSTFLAGS="${RUSTFLAGS:+"${RUSTFLAGS:?} "}-C link-arg=-s"
readonly RUSTFLAGS

error_report_fd="3"
readonly error_report_fd

case "${error_report_fd:?}" in
  ("0"|"1"|"2")
    "echo" \
      "Error report file descriptor set to a predefined file descriptor \
number!" \
      >&2
    ;;
  (*)
    case "${error_report_fd##[[:digit:]]}" in
      ("") ;;
      (*)
        "echo" \
          "Error report file descriptor has to be a non-negative number!" \
          >&2
        ;;
    esac
    ;;
esac

error() {
  case "${#:?}" in
    ("1")
      "echo" \
        "${1:?}" \
        >&2

      if "is_error_report_fd_open"
      then
        "echo" \
          "${1:?}" \
          >&"${error_report_fd:?}"
      fi
      ;;
    (*)
      "error" "\"error\" function requires exactly one argument! Number of \
passed arguments: ${#:?}."
      ;;
  esac

  exit "1"
}

is_error_report_fd_open() (
  { { :; } >&"${error_report_fd:?}"; } 2>&-
)

open_error_report_fd() {
  if ! eval "exec ${error_report_fd:?}>\"/artifacts/error-report.txt\""
  then
    "error" "Failed to open error report file descriptor!"
  fi
}

close_error_report_fd() {
  if ! eval "exec ${error_report_fd:?}>&-"
  then
    "error" "Failed to close error report file descriptor!"
  fi
}

if ! "is_error_report_fd_open"
then
  "open_error_report_fd"
fi

assert_arguments_count() (
  expected_number_of_arguments="${1:?}"
  readonly expected_number_of_arguments

  shift

  if ! shift \
    "${expected_number_of_arguments%"+"}" \
    2>&"${error_report_fd:?}"
  then
    "error" "Got less than the expected at least \
${expected_number_of_arguments%"+"} arguments!"
  fi

  case "${expected_number_of_arguments:?}" in
    (*"+") ;;
    (*)
      case "${#:?}" in
        ("0") ;;
        (*)
          "error" "Got more than the expected exactly \
${expected_number_of_arguments:?} arguments!"
          ;;
      esac
      ;;
  esac
)

# shellcheck disable=SC2120
is_privileged() (
  "assert_arguments_count" \
    "0" \
    "${@}"

  user_id="$(
    "id" \
      --user \
      2>&"${error_report_fd:?}"
  )"
  readonly user_id

  real_user_id="$(
    "id" \
      --real \
      --user \
      2>&"${error_report_fd:?}"
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

exec_in_child_shell() (
  script="${1?}"
  readonly script

  expected_number_of_arguments="${2:?}"

  shift "2"

  "assert_arguments_count" \
    "${expected_number_of_arguments:?}" \
    "${@}"

  if "is_error_report_fd_open"
  then
    redirect_fd="${error_report_fd:?}"
  else
    redirect_fd="2"
  fi

  "sh" \
    -c \
    -eu \
    "${script?}" \
    "sh" \
    "${@}" \
    2>&"${redirect_fd:?}"
)

clean_dir_contents() (
  # shellcheck disable=SC2016
  script='cd "/${1:?}"

find \
  "." \
  -depth \
  -path "./?*" \
  "!" -path "./?*/?*" \
  "(" \
  -exec "rm" "-f" "-R" "{}" ";" \
  -o \
  -exec "kill" "${$:?}" ";" \
  ")"'

  "exec_in_child_shell" \
    "${script:?}" \
    "1" \
    "${@}"
)

# shellcheck disable=SC2120
make_ephemeral_directories() (
  "assert_arguments_count" \
    "0" \
    "${@}"

  for directory in \
    "target" \
    "rootless-artifacts"
  do
    if ! "mkdir" \
      -m "0557" \
      "/${directory:?}" \
      2>&"${error_report_fd:?}"
    then
      "error" "Failed to create \"/${directory:?}\" directory!"
    fi
  done
)

run_unprivileged() {
  "assert_arguments_count" \
    "1+" \
    "${@}"

  if ! RUN_UNPRIVILEGED="1" \
    "setpriv" \
    --reuid="1000" \
    --regid="1000" \
    --clear-groups \
    --inh-caps="-all" \
    --no-new-privs \
    "${0:?}" \
    "${@:?}"
  then
    "error" "Failed to run with dropped privileges!"
  fi
}

# shellcheck disable=SC2120
remove_cargo_target_dir() {
  "assert_arguments_count" \
    "0" \
    "${@}"

  if ! "rm" \
    -R \
    "/${CARGO_TARGET_DIR:?}" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to remove Cargo's target directory!"
  fi
}

copy_dir_contents() (
  # shellcheck disable=SC2016
  script='cd "/${1:?}"

case "${2:?}" in
  (*"{}"*)
    "echo" \
      "Target directory contains reserved combination, \"{}\", in its path!" \
      >&2

    exit "1"
    ;;
esac

find \
  "." \
  -path "./?*" \
  "!" -path "./?*/?*" \
  "(" \
  -exec "cp" "-R" "{}" "/${2:?}/" ";" \
  -o \
  -exec "kill" "${$:?}" ";" \
  ")"'

  if ! "exec_in_child_shell" \
    "${script:?}" \
    "2" \
    "${@}"
  then
    "error" "Failed to copy contents of \"/${1:?}\" to \"/${2:?}\"!"
  fi
)

recursively_take_ownership_dir() {
  "assert_arguments_count" \
    "1" \
    "${@}"

  if ! "chown" \
    -R \
    "0:0" \
    "/${1:?}" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to recursively take ownership of \"/${1:?}\"!"
  fi
}

# shellcheck disable=SC2120
set_permissions_of_contents() {
  # shellcheck disable=SC2016
  script='cd "/${1:?}"

unexpected_file_type_script="\
\"echo\" \"Unexpected file type!\" >&2

\"kill\" \"\${1:?}\""

find \
  "." \
  -path "./?*" \
  "(" \
  "(" \
  -type "d" \
  -exec "chmod" "0755" "{}" ";" \
  ")" \
  -o \
  "(" \
  -type "f" \
  -exec "chmod" "0644" "{}" ";" \
  ")" \
  -o \
  -exec "sh" "-c" "${unexpected_file_type_script:?}" "sh" "${$:?}" ";" \
  ")"'

  if ! "exec_in_child_shell" \
    "${script:?}" \
    "1" \
    "${@:?}"
  then
    "error" "Failed to set permissions of contents of \"${1:?}\"!"
  fi
}

move_dir_contents() (
  # shellcheck disable=SC2016
  script='cd "/${1:?}"

case "${2:?}" in
  (*"{}"*)
    "echo" \
      "Target directory contains reserved combination, \"{}\", in its path!" \
      >&2

    exit "1"
    ;;
esac

find \
  "." \
  -depth \
  -path "./?*" \
  "!" -path "./?*/?*" \
  "(" \
  -exec "mv" "{}" "/${2:?}" ";" \
  -o \
  -exec "kill" "${$:?}" ";" \
  ")"'

  if ! "exec_in_child_shell" \
    "${script:?}" \
    "2" \
    "${@}"
  then
    "error" "Failed to move contents of \"${1:?}\" to \"${2:?}\"!"
  fi
)

rerun_as_unprivileged() {
  "assert_arguments_count" \
    "1+" \
    "${@}"

  if "is_privileged"
  then
    case "${RUN_UNPRIVILEGED+"1"}" in
      ("1")
        "error" "Running in privileged mode while \"RUN_UNPRIVILEGED\" is set!"
        ;;
    esac

    build_configuration="/build-configuration"
    readonly build_configuration

    protocol_json="/${build_configuration:?}/protocol.json"
    readonly protocol_json

    topology_json="/${build_configuration:?}/topology.json"
    readonly topology_json

    "close_error_report_fd"

    if ! "clean_dir_contents" "/artifacts"
    then
      "error" "Failed to clean up \"/artifacts\" during preparations!"
    fi

    "open_error_report_fd"

    "make_ephemeral_directories"

    if "test" \
      -r "${protocol_json:?}" \
      -o \
      -r "${topology_json:?}" \
      2>&"${error_report_fd:?}"
    then
      if ! protocol="$(
        "cat" \
          "${protocol_json:?}" \
          2>&"${error_report_fd:?}"
      )"
      then
        "error" "Failed to read protocol definition from file!"
      fi
      readonly protocol
      : "${protocol:?}"

      if ! topology="$(
        "cat" \
          "${topology_json:?}" \
          2>&"${error_report_fd:?}"
      )"
      then
        "error" "Failed to read topology definition from file!"
      fi
      readonly topology
      : "${topology:?}"

      "run_unprivileged" \
        "${@:?}" \
        "${protocol:?}" \
        "${topology:?}"
    else
      "run_unprivileged" "${@:?}"
    fi

    "remove_cargo_target_dir"

    "copy_dir_contents" \
      "/labels" \
      "/rootless-artifacts"

    "recursively_take_ownership_dir" "/rootless-artifacts"

    "move_dir_contents" \
      "/rootless-artifacts" \
      "/artifacts"

    if ! "rmdir" \
      "/rootless-artifacts" \
      2>&"${error_report_fd:?}"
    then
      "error" "Failed to remove rootless artifacts directory!"
    fi

    exit "0"
  else
    export RUSTFLAGS

    : "${RUN_UNPRIVILEGED?}"
    unset RUN_UNPRIVILEGED
  fi
}

"rerun_as_unprivileged" "${@}"

check_groups() {
  group_ids="$(
    "id" \
      --groups \
      2>&"${error_report_fd:?}"
  )"
  readonly group_ids
  real_group_ids="$(
    "id" \
      --real \
      --groups \
      2>&"${error_report_fd:?}"
  )"
  readonly real_group_ids

  for ids in \
    "${group_ids:?}" \
    "${real_group_ids:?}"
  do
    case "${ids:?}" in
      ("0"|"0"[![:digit:]]*|*[![:digit:]]"0"[![:digit:]]*|*[![:digit:]]"0")
        "error" "Running with non-root user IDs but group IDs contain the root \
group!"
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
      --locked \
      2>&"${error_report_fd:?}"
    then
      "error" "Dependencies are out of date!"
    fi
    ;;
esac

if RELEASE_VERSION="$(
  "cat" \
    "/labels/release-version.txt" \
    2>&"${error_report_fd:?}"
)"
then
  readonly RELEASE_VERSION
  : \
    "${RELEASE_VERSION:?"Release version cannot be null!"}" \
    2>&"${error_report_fd:?}"
  export RELEASE_VERSION
else
  "error" "Failed to read release version!"
fi

if cosmwasm_capabilities="$(
  "cat" \
    "/configuration/cosmwasm_capabilities" \
    2>&"${error_report_fd:?}"
)"
then
  readonly cosmwasm_capabilities
else
  "error" "Failed to read available CosmWasm capabilities!"
fi

if ! build_profile="${1:?}" \
  2>&"${error_report_fd:?}"
then
  "error" "Passing build profile as first parameter is required!"
fi
readonly build_profile
shift


build_profiles_directory="/configuration/build-profiles/"
readonly build_profiles_directory

if mapped_build_profile="$(
  "cat" \
    "${build_profiles_directory:?}/${build_profile:?}" \
    2>&"${error_report_fd:?}"
)"
then
  readonly mapped_build_profile

  if ! : \
    "${mapped_build_profile:?}" \
    2>&"${error_report_fd:?}"
  then
    "error" "Mapped build profile cannot be null!"
  fi
else
  if build_profiles="$(
    "ls" \
      -1 \
      "${build_profiles_directory:?}" \
      2>&"${error_report_fd:?}"
  )"
  then
    case "${build_profiles?}" in
      ("")
        "error" "No build profiles present!"
        ;;
    esac

    build_profiles_pretty=""

    while read -r build_profiles_entry
    do
      build_profiles_pretty="${build_profiles_pretty:+"${build_profiles_pretty:?}
"}* ${build_profiles_entry:?}"
    done <<EOF
${build_profiles?}
EOF

    "error" "Failed to read build profile mapping!

Existing profiles:
${build_profiles_pretty?}"
  else
    "error" "Failed to read available build profiles!"
  fi
fi

if ! max_binary_size="$(
  "cat" \
    "/configuration/${build_profile:?}-max-binary-size" \
    2>&"${error_report_fd:?}"
)"
then
  "error" "Failed to read max binary size for build profile!"
fi
readonly max_binary_size
if ! : \
  "${max_binary_size:?}" \
  2>&"${error_report_fd:?}"
then
  "error" "Maximum binary size cannot be null!"
fi

working_directory="$(
  if ! "pwd" 2>&"${error_report_fd:?}"
  then
    "error" "Failed to retrieve current directory's path!"
  fi
)"
working_directory="$(
  if ! "basename" \
    "${working_directory:?}" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to retrieve name of current directory!"
  fi
)"
readonly working_directory
if ! : \
  "${working_directory:?}" \
  2>&"${error_report_fd:?}"
then
  "error" "Working directory's name cannot be an empty string!"
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
    case "${#:?}" in
      ("0")
        "error" "Expected protocol and topology configurations!"
        ;;
      ("1")
        "error" "Got only one parameter! Most probably cause is an internal \
error!"
        ;;
      ("2") ;;
      (*)
        "error" "Got more than the two expected parameters, containing \
protocol and topology! Got ${#:?} arguments!"
        ;;
    esac

    if ! protocol="${1:?}" 2>&"${error_report_fd:?}"
    then
      "error" "Protocol definition JSON cannot be empty!"
    fi
    if ! topology="${2:?}" 2>&"${error_report_fd:?}"
    then
      "error" "Topology definition JSON cannot be empty!"
    fi

    shift "2"

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

      if ! : \
        "${protocol:?}" \
        2>&"${error_report_fd:?}"
      then
        "error" "Protocol definition JSON cannot be empty!"
      fi
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
        2>&"${error_report_fd:?}" \
        <<EOF
${topology:?}
EOF
    )"
    then
      unset topology

      readonly dex_type
      if ! : \
        "${dex_type:?}" \
        2>&"${error_report_fd:?}"
      then
        "error" "DEX type cannot be null!"
      fi
    else
      "error" "Failed to get DEX type from topology describing JSON file!"
    fi
    ;;
  (*)
    "error" "Current directory corresponds to an unknown workspace!"
    ;;
esac
readonly dex_type
if ! : \
  "${dex_type?}" \
  2>&"${error_report_fd:?}"
then
  "error" "DEX type variable not set! Most likely an internal error!"
fi

if ! contracts_count="$(
  "cat" \
    "/configuration/${working_directory:?}-contracts-count" \
    2>&"${error_report_fd:?}"
)"
then
  "error" "Failed to read expected contracts count configuration for workspace!"
fi
readonly contracts_count
if ! : \
  "${contracts_count:?}" \
  2>&"${error_report_fd:?}"
then
  "error" "Contracts count cannot be null!"
fi

CURRENCIES_BUILD_REPORT="/rootless-artifacts/currencies.build.log"
readonly CURRENCIES_BUILD_REPORT
export CURRENCIES_BUILD_REPORT

for tag in \
  "@agnostic" \
  "${dex_type:+"dex-${dex_type:?}"}"
do
  case "${tag?}" in
    ("")
      continue
      ;;
  esac

  if ! "cargo" \
    -- \
    "each" \
    --tag "build" \
    --tag "${tag:?}" \
    "run" \
    --exact \
    -- \
    --quiet \
    "build" \
    --profile "${mapped_build_profile:?}" \
    --lib \
    --frozen \
    --target "wasm32-unknown-unknown" \
    --target-dir "/${CARGO_TARGET_DIR:?}" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to build contracts in workspace tagged with \"${tag}\"!"
  fi

  "echo"
done

if ! cargo_output_directory="/${CARGO_TARGET_DIR:?}/wasm32-unknown-unknown/\
${mapped_build_profile:?}/" 2>&"${error_report_fd:?}"
then
  "error" "Failed to determine the path to the build output directory!"
fi 
readonly cargo_output_directory

files="$(
  if ! cd \
    "/${cargo_output_directory:?}" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to change the working directory to \
\"${cargo_output_directory:?}\"!"
  fi

  if ! "find" \
    "." \
    -type "f" \
    -name "*.wasm" \
    -path "./?*" \
    "!" -path "./?*/?*" \
    -print \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to list compiled WebAssembly binaries!"
  fi
)"
if ! files="$(
  "sort" \
    2>&"${error_report_fd:?}" \
    <<EOF
${files:?}
EOF
)"
then
  "error" "Failed to sort output directory files' paths via \"sort\"!"
fi
readonly files

if ! file_count="$(
  "wc" \
    -l \
    2>&"${error_report_fd:?}" \
    <<EOF
${files:?}
EOF
)"
then
  "error" "Failed to retrieve the output directories' binaries count via \
\"wc\"!"
fi
readonly file_count

case "${file_count:?}" in
  ("0")
    "error" "Build produced no output! Expected ${contracts_count:?} contracts!"
    ;;
  ("${contracts_count:?}") ;;
  (*)
    "error" "Expected ${contracts_count:?} contracts, got ${file_count:?}!"
    ;;
esac

while read -r wasm_path
do
  if ! wasm_name="$(
    "basename" \
      "${wasm_path:?}" \
      2>&"${error_report_fd:?}"
  )"
  then
    "error" "Failed to extract basename from artifact file's path!"
  fi

  "echo" "Optimizing: ${wasm_name:?}"

  if "wasm-opt" \
    -Os \
    --inlining-optimizing \
    --signext-lowering \
    -o "/rootless-artifacts/${wasm_name:?}" \
    "/${cargo_output_directory}/${wasm_path:?}" \
    2>&"${error_report_fd:?}"
  then
    "echo" "Finished optimizing: ${wasm_name:?}"
  else
    "error" "Failed to run \"wasm-opt\" on \"${wasm_name:?}\"!"
  fi

  "echo"
done \
  <<EOF
${files:?}
EOF

large_files="$(
  if ! cd \
    "/rootless-artifacts" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to change the working directory to \"/rootless-artifacts\"!"
  fi

  if ! "find" \
    "." \
    -type "f" \
    -name "*.wasm" \
    -size "+${max_binary_size:?}" \
    -printf "%f - %s bytes\n" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to retrieve list of artifacts that are above allowed size!"
  fi
)"
readonly large_files

case "${large_files?}" in
  ("") ;;
  (*)
    "error" "### These files are larger than the allowed limit, \
${max_binary_size:?}:
${large_files:?}"
  ;;
esac

while read -r wasm_path
do
  if ! "cosmwasm-check" \
    --available-capabilities "${cosmwasm_capabilities?}" \
    "/rootless-artifacts/${wasm_path:?}" \
    2>&"${error_report_fd:?}"
  then
    "error" "CosmWasm check failed for \"${wasm_path:?}\"!"
  fi

  "echo"
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

    if ! "mkdir" \
      -m "0755" \
      "/${outputs_directory:?}"
    then
      "error" "Failed to create directory for build outputs!"
    fi

    while read -r build_output_package
    do
      build_output_directories="$(
        if ! cd \
          "/${cargo_output_directory:?}" \
          2>&"${error_report_fd:?}"
        then
          "error" "Failed to change working directory to \
\"/${cargo_output_directory:?}\"!"
        fi

        if ! "find" \
          "." \
          -type "d" \
          -path "./build/${build_output_package:?}-????????????????/out" \
          2>&"${error_report_fd:?}"
        then
          "error" "Failed to list build script binaries' output directory!"
        fi
      )"

      case "${build_output_directories?}" in
        ("")
          "error" "Retrieved list of build script output directories doesn't \
contain \"${build_output_package:?}-<FINGERPRINT>\" directories!"
          ;;
        (*)
          while read -r build_output
          do
            if ! "cp" \
              -R \
              "/${cargo_output_directory:?}/${build_output:?}/" \
              "/${outputs_directory:?}/" \
              2>&"${error_report_fd:?}"
            then
              "error" "Failed to copy recursively \
\"/${cargo_output_directory:?}/${build_output:?}/\" to \
\"/${outputs_directory:?}/\"!"
            fi
          done \
            <<EOF
${build_output_directories:?}
EOF
      esac
    done \
      <<EOF
${build_output_packages?}
EOF
    ;;
esac

checksum="$(
  if ! cd \
    "/rootless-artifacts/" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to change working directory to \"/rootless-artifacts/\"!"
  fi

  if ! "find" \
    "." \
    -name "?*.wasm" \
    -exec "sha256sum" "--" "{}" ";" \
    2>&"${error_report_fd:?}"
  then
    "error" "Failed to calculate artifact checksums!"
  fi
)"
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
