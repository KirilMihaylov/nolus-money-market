#!/bin/sh

RUSTFLAGS="-C link-arg=-s ${RUSTFLAGS}"

cd "/code/" || exit

for signal in EXIT HUP INT QUIT TERM
do
  trap "rm -rf \"/target/\"*" "$signal"
done

rust_version="$(cat "/rust-version")"

if "[" "${CHECK_DEPENDENCIES_UPDATED}" != "false" ]
then
  cargo "+${rust_version}" update --locked

  if "[" "${?}" -ne 0 ]
  then
      echo "[ERROR] \"Cargo.lock\" file is either missing or is out-of-date!"

      exit 1
  fi
fi

rm -rf "/target/"*

rm -rf "/artifacts/"*

rm -rf "/temp-artifacts/"*

if "[" -z "${NETWORK}" ]
then
  echo "[ERROR] Environment variant \"NETWORK\", indicating filter group, \
isn't set!"

  exit 1
fi

if "[" -z "${PROTOCOL}" ]
then
  echo "[ERROR] Environment variant \"PROTOCOL\", indicating filter group, \
isn't set!"

  exit 1
fi

if "[" -z "${PROFILE}" ]
then
  echo "[ERROR] Environment variant \"PROFILE\", indicating filter group, \
isn't set!"

  exit 1
fi

cargo "+${rust_version}" -- each --group "build" --group "${NETWORK}" --group "${PROTOCOL}" \
  run --exact -- build --profile "${PROFILE}" --lib --locked \
  --target "wasm32-unknown-unknown" --target-dir "/target/"

if "[" "${?}" -ne 0 ]
then
    echo "[ERROR] Cargo exited with non-zero status code!"

    exit 1
fi

output_directory="/target/wasm32-unknown-unknown/${PROFILE}/"

file_count="$(
  find "${output_directory}" -type f -name "*.wasm" -exec "printf" "." ";" \
    | wc -c | tr -d "\\n"
)"

if "[" "${file_count}" -eq 0 ]
then
  echo "Build produced no output!"

  exit 1
fi

for wasm_path in $(find "${output_directory}" -maxdepth 1 -name "*.wasm" | sort)
do
  wasm_name="$(basename "${wasm_path}")"

  echo "Optimizing: ${wasm_name}"

  wasm-opt -Os --signext-lowering -o "/temp-artifacts/${wasm_name}" \
    "${wasm_path}"

  if "[" ${?} -ne 0 ]
  then
      echo "[ERROR] \"wasm-opt\" exited with non-zero status code while being \
ran against \"${wasm_name}\"!"

      exit 1
  fi

  echo "Finished optimizing: ${wasm_name}"
done

cp -t "/artifacts/" "/rust-version"

cp -t "/artifacts/" "/binaryen-version"

mv -t "/artifacts/" "/temp-artifacts/"*".wasm"

printf "\nChecksums:\n"

sha256sum -- "/artifacts/"*".wasm" | tee "/artifacts/checksums.txt"
