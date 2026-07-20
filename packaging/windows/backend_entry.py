"""PyInstaller entry point for the toolkit-free backend process."""

from pybmap.backend.worker import run


if __name__ == "__main__":
    raise SystemExit(run())
