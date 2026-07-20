# bosectl Windows desktop development

Stage 1 provides the Tauri 2, React, TypeScript, and Python-sidecar foundation.
The IPC and renderer command contracts are frozen in
`../docs/windows-gui-protocol.md`. Bluetooth operations intentionally return
`BACKEND_UNAVAILABLE` until the Stage 2 sidecar and Rust supervisor are added;
`hello` and `shutdown` already run end to end in the Python worker.

## Prerequisites

- Windows 10 or 11 with Evergreen WebView2
- Node.js 22.12 or newer and npm
- Rust 1.77.2 or newer with the MSVC Windows target
- Python 3.8 or newer for backend development

## Commands

From `desktop/`:

```powershell
npm install
npm run dev
npm test
npm run build
npm run tauri dev
npm run tauri build
```

From `python/`:

```powershell
python -m pytest
python -m pybmap.backend
```

To smoke-test the sidecar, write one compact JSON request per line. Start with
`{"v":1,"id":"1","method":"hello","params":{}}` and finish with
`{"v":1,"id":"2","method":"shutdown","params":{}}`.

The renderer imports native APIs only through `src/api.ts`. Do not add shell,
filesystem, path, process, or arbitrary event capabilities. The Rust host accepts
typed domain inputs and validates them before the sidecar boundary.
