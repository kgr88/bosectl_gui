# Windows release packaging

The release pipeline freezes the Python backend in one-folder mode, smoke-tests
the frozen executable, stages it under Tauri's target-triple sidecar name, runs
the software-only test suites, and builds a current-user NSIS installer. The
installer uses the Evergreen WebView2 download bootstrapper, so it does not
require administrator access or bundle a fixed browser runtime.

Prerequisites are a 64-bit Windows 10 or 11 release machine with Python 3.12,
PyInstaller, Node.js/Corepack, Rust/MSVC, and the normal project dependencies:

```powershell
python -m pip install pyinstaller pytest
corepack pnpm --dir desktop install --frozen-lockfile
```

Build from the repository root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File packaging/windows/build-release.ps1
```

Use `-PythonExecutable C:\path\to\python.exe` when Python is not on `PATH`.
The generated installer and its SHA-256 file are placed below
`desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/`.

For a signed release, import the release certificate into the build user's
certificate store and set `BOSECTL_CERTIFICATE_THUMBPRINT` before running the
same command. Tauri then signs the application, sidecar, uninstaller, and
installer with SHA-256 and timestamps them. Do not publish an unsigned build.

Before publishing, test the installer on clean Windows 10 and Windows 11 VMs:

1. Install as a standard user and confirm no Python, Node.js, Rust, or elevation
   prompt is required.
2. Verify first launch, manual/discovered connection, normal taskbar minimizing,
   closing to the tray, the numeric battery icon and percentage tooltip, reopening
   controls from the tray, disconnect, and clean shutdown with a paired headset.
3. Upgrade from the previous published version and confirm settings and the tray
   shortcut continue to work.
4. Uninstall, confirm the application and sidecar processes are gone, and confirm
   installed files and shortcuts are removed.
5. Measure the complete release process tree in each state from `implementation.md`.
6. Verify Authenticode signatures and compare the published SHA-256 checksum.

Hardware, installer upgrade/uninstall behavior, resource targets, and clean-VM
runtime independence cannot be claimed from the software-only build pipeline;
record those results in the release notes for each candidate.
