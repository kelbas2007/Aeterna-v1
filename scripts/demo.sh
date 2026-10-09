#!/usr/bin/env bash
# Each demonstration learns and restores in separate processes. Keep the
# checkpoints in a fresh directory so earlier knowledge is never overwritten.
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"

cargo build --locked --release --bins --examples
example_bin_dir="$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')/release/examples"
checkpoint_dir="$(mktemp -d "${TMPDIR:-/tmp}/aeterna-demos.XXXXXX")"

for example in online_learning learned_rules partial_observation expanded_rules inverse_inference adaptive_learning uncertain_learning; do
    checkpoint_path="$checkpoint_dir/$example.json"
    "$example_bin_dir/$example" learn "$checkpoint_path"
    "$example_bin_dir/$example" run "$checkpoint_path"
done
perception_bin="${example_bin_dir%/examples}/perception"
"$perception_bin" train tests/data/digits.csv "$checkpoint_dir/perception.json"
python3 - "$checkpoint_dir/digit.pgm" <<'PY'
import sys
from pathlib import Path
pixels = Path("tests/data/digits.csv").read_text().splitlines()[0].split(",")[:64]
Path(sys.argv[1]).write_text("P2\n8 8\n16\n" + " ".join(str(int(float(x))) for x in pixels) + "\n")
PY
"$perception_bin" recognize "$checkpoint_dir/perception.json" "$checkpoint_dir/digit.pgm"
"$perception_bin" correct "$checkpoint_dir/perception.json" "$checkpoint_dir/digit.pgm" 0 "$checkpoint_dir/perception-corrected.json"
printf 'Saved demonstration checkpoints: %s\n' "$checkpoint_dir"
