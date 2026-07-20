"""Run the development backend with ``python -m pybmap.backend``."""

from .worker import run


if __name__ == "__main__":
    raise SystemExit(run())
