"""Fail when the release version differs between the project files."""

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read_text(relative_path: str) -> str:
    return (ROOT / relative_path).read_text(encoding="utf-8")


def read_versions() -> dict[str, str]:
    package_json = json.loads(read_text("desktop/package.json"))
    tauri_conf = json.loads(read_text("desktop/src-tauri/tauri.conf.json"))
    cargo = tomllib.loads(read_text("desktop/src-tauri/Cargo.toml"))
    pyproject = tomllib.loads(read_text("python/pyproject.toml"))
    backend_version = read_text("packaging/windows/backend-version.txt")
    return {
        "desktop/package.json": package_json["version"],
        "desktop/src-tauri/tauri.conf.json": tauri_conf["version"],
        "desktop/src-tauri/Cargo.toml": cargo["package"]["version"],
        "python/pyproject.toml": pyproject["project"]["version"],
        "backend-version.txt FileVersion": re.search(
            r"StringStruct\(u'FileVersion', u'([^']+)'\)", backend_version
        ).group(1),
        "backend-version.txt ProductVersion": re.search(
            r"StringStruct\(u'ProductVersion', u'([^']+)'\)", backend_version
        ).group(1),
    }


def main() -> int:
    versions = read_versions()
    for path, version in versions.items():
        print(f"{path}: {version}")
    if len(set(versions.values())) != 1:
        print("Version mismatch: all files must use one version.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
