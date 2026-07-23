# Run from the repository root with:
#   python -m PyInstaller --noconfirm --clean packaging/windows/pyinstaller.spec
from pathlib import Path

repository = Path(SPECPATH).parents[1]
python_root = repository / "python"

analysis = Analysis(
    [str(repository / "packaging" / "windows" / "backend_entry.py")],
    pathex=[str(python_root)],
    binaries=[],
    datas=[],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
)
pyz = PYZ(analysis.pure)
executable = EXE(
    pyz,
    analysis.scripts,
    [],
    exclude_binaries=True,
    name="bosectl-backend",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    console=True,
    icon=str(repository / "desktop" / "src-tauri" / "icons" / "icon.ico"),
    version=str(repository / "packaging" / "windows" / "backend-version.txt"),
    contents_directory="bosectl-backend-lib",
)
collection = COLLECT(
    executable,
    analysis.binaries,
    analysis.datas,
    strip=False,
    upx=False,
    name="bosectl-backend",
)
