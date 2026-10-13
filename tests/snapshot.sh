#!/bin/sh
# Build the plugin wasm and require render() bytes to match shots/*.ansi.txt.
#
# zellij-plugin-snapshot is a command. [dependencies] and [dev-dependencies]
# link a library into the wasm or into `cargo test`, so they leave this binary
# uninstalled. Install the pinned release first:
#   rustup run "$(rustup show active-toolchain | awk '{print $1}')" cargo install zellij-plugin-snapshot --version 0.2.2 --locked
# https://github.com/fulldecent/zellij-plugin-snapshot
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$root"

# `rustup run` selects the Cargo from rust-toolchain.toml. On GitHub Actions,
# `rustup which cargo` was the default stable toolchain, which had no
# wasm32-wasip1 standard library. `rustup run` does not put that toolchain's
# rustc on PATH. tools/rustc-wrapper, configured as Cargo's rustc-wrapper,
# runs the matching rustc when PATH's compiler has no wasm32-wasip1 sysroot.
run_cargo() {
  if command -v rustup >/dev/null 2>&1; then
    rustup=$(command -v rustup)
    toolchain=$("$rustup" show active-toolchain)
    toolchain=${toolchain%% *}
    "$rustup" run "$toolchain" cargo "$@"
  else
    cargo "$@"
  fi
}

# `cargo install` places zellij-plugin-snapshot in ~/.cargo/bin.
PATH="${HOME}/.cargo/bin:${PATH}"
export PATH

if ! command -v zellij-plugin-snapshot >/dev/null 2>&1; then
  echo "zellij-plugin-snapshot 0.2.2 is not on PATH." >&2
  echo 'rustup run "$(rustup show active-toolchain | awk "{print \$1}")" cargo install zellij-plugin-snapshot --version 0.2.2 --locked' >&2
  exit 1
fi

# .cargo/config.toml already selects wasm32-wasip1 for a plain `cargo build`.
# Name the target here so this script still builds the plugin wasm if that
# default is removed. `cargo test` cannot do this job: it locks the target
# directory, and it runs the host harness rather than the plugin wasm.
if command -v rustup >/dev/null 2>&1; then
  "$(command -v rustup)" target add wasm32-wasip1
fi
run_cargo build --release --locked --target wasm32-wasip1

# zellij-plugin-snapshot writes `{name}.ansi.txt` from the YAML `name` field
# (file stem if `name` is omitted). Compare those generated files, not the
# yaml filename.
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT

failed=0
matched=0
for yaml in shots/*.yaml; do
  [ -f "$yaml" ] || continue
  matched=$((matched + 1))
  find "$out" -mindepth 1 -delete
  zellij-plugin-snapshot "$yaml" --out "$out"
  produced=0
  for ansi in "$out"/*.ansi.txt; do
    [ -f "$ansi" ] || continue
    produced=$((produced + 1))
    base=$(basename "$ansi")
    if ! diff -u "shots/$base" "$ansi"; then
      stem=${base%.ansi.txt}
      echo "Render bytes differ from shots/$base." >&2
      echo "When that change is intended:" >&2
      echo "  zellij-plugin-snapshot $yaml --out /tmp/shots" >&2
      echo "  cp /tmp/shots/${stem}.ansi.txt shots/${stem}.ansi.txt" >&2
      # screenshot.svg is the picture linked from the repository root.
      # The other pictures live beside their YAML scripts.
      if [ "$stem" = screenshot ]; then
        echo "  cp /tmp/shots/${stem}.svg screenshot.svg" >&2
      else
        echo "  cp /tmp/shots/${stem}.svg shots/${stem}.svg" >&2
      fi
      failed=1
    fi
  done
  if [ "$produced" -eq 0 ]; then
    echo "zellij-plugin-snapshot wrote no .ansi.txt for $yaml" >&2
    failed=1
  fi
done
if [ "$matched" -eq 0 ]; then
  echo "no shots/*.yaml files" >&2
  exit 1
fi
if [ "$failed" -ne 0 ]; then
  exit 1
fi
