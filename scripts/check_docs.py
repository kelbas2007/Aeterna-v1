#!/usr/bin/env python3
"""Check tracked and untracked Markdown file links without network requests."""

import re
import subprocess
from pathlib import Path
from urllib.parse import unquote, urlsplit


def main():
    root = Path(__file__).resolve().parents[1]
    filenames = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "*.md"],
        cwd=root,
    ).decode().split("\0")
    errors = []
    checked = 0
    for filename in sorted(set(filter(None, filenames))):
        document = root / filename
        if not document.is_file():
            continue
        checked += 1
        for number, line in enumerate(document.read_text().splitlines(), 1):
            for target in re.findall(r"!?\[[^\]\n]*\]\(\s*(<[^>\n]+>|[^\s)\n]+)", line):
                destination = urlsplit(target.strip("<>"))
                if destination.scheme or destination.netloc or not destination.path:
                    continue
                path = unquote(destination.path)
                resolved = (root / path.lstrip("/")) if path.startswith("/") else document.parent / path
                if not resolved.exists():
                    errors.append(f"{filename}:{number}: missing {path}")
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"Markdown links: {checked} documents checked, no missing local targets")


if __name__ == "__main__":
    main()
