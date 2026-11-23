#!/usr/bin/env python3
"""Verify or reconstruct the complete committed autonomous inquiry archive."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile


DIRECTORY = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def verify(manifest, spool=None):
    archive_hash = hashlib.sha256()
    total = 0
    for index, chunk in enumerate(manifest["chunks"], 1):
        name = chunk["name"]
        require(Path(name).name == name and name.endswith(f".part-{index:03d}"),
                "Invalid or unordered chunk filename")
        path = DIRECTORY / name
        require(path.is_file() and not path.is_symlink(), f"Missing regular chunk: {name}")
        expected_size = chunk["bytes"]
        require(0 < expected_size <= manifest["maximum_chunk_bytes"], "Invalid chunk size")
        digest = hashlib.sha256()
        git_digest = hashlib.sha1(f"blob {expected_size}\0".encode("ascii"))
        size = 0
        with path.open("rb") as source:
            for block in iter(lambda: source.read(1024 * 1024), b""):
                size += len(block)
                require(size <= expected_size, f"Oversized chunk: {name}")
                digest.update(block)
                git_digest.update(block)
                archive_hash.update(block)
                if spool is not None:
                    spool.write(block)
        require(size == expected_size, f"Wrong chunk length: {name}")
        require(digest.hexdigest() == chunk["sha256"], f"SHA256 mismatch: {name}")
        require(git_digest.hexdigest() == chunk["git_blob_sha1"], f"Git blob mismatch: {name}")
        total += size
    require(total == manifest["archive_bytes"], "Reconstructed archive length differs")
    require(archive_hash.hexdigest() == manifest["archive_sha256"], "Reconstructed archive SHA256 differs")
    return {"verified": True, "chunks": len(manifest["chunks"]), "bytes": total,
            "sha256": archive_hash.hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="New archive file; existing paths are never overwritten")
    parser.add_argument("--verify-only", action="store_true", help="Verify all chunks and the complete archive without writing")
    args = parser.parse_args()
    require(args.verify_only != (args.output is not None), "Choose exactly one of --output or --verify-only")
    manifest = json.loads((DIRECTORY / "manifest.json").read_text(encoding="utf-8"))
    if args.verify_only:
        result = verify(manifest)
    else:
        output = args.output.expanduser().absolute()
        require(output.parent.is_dir(), "Output directory must already exist")
        require(not output.exists() and not output.is_symlink(), "Output already exists")
        with tempfile.TemporaryFile(dir=output.parent) as spool:
            result = verify(manifest, spool)
            spool.seek(0)
            identity = None
            try:
                with output.open("xb") as destination:
                    stat = os.fstat(destination.fileno())
                    identity = (stat.st_dev, stat.st_ino)
                    shutil.copyfileobj(spool, destination, 1024 * 1024)
                    destination.flush()
                    os.fsync(destination.fileno())
            except BaseException:
                if identity is not None:
                    try:
                        stat = output.lstat()
                        if (stat.st_dev, stat.st_ino) == identity:
                            output.unlink()
                    except OSError:
                        pass
                raise
        result["output"] = str(output)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise SystemExit(f"Archive verification failed: {error}")
