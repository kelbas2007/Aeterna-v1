#!/usr/bin/env bash
# Each demonstration learns and restores in separate processes. Keep the
# checkpoints in a fresh directory so earlier knowledge is never overwritten.
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"

cargo build --locked --release --examples
example_bin_dir="$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')/release/examples"
checkpoint_dir="$(mktemp -d "${TMPDIR:-/tmp}/aeterna-demos.XXXXXX")"

for example in online_learning learned_rules partial_observation expanded_rules inverse_inference adaptive_learning; do
    checkpoint_path="$checkpoint_dir/$example.json"
    "$example_bin_dir/$example" learn "$checkpoint_path"
    "$example_bin_dir/$example" run "$checkpoint_path"
done
printf 'Saved demonstration checkpoints: %s\n' "$checkpoint_dir"
