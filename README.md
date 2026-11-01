# bosectl

[![CI](https://github.com/kgr88/bosectl_gui/actions/workflows/ci.yml/badge.svg)](https://github.com/kgr88/bosectl_gui/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/kgr88/bosectl_gui)](https://github.com/kgr88/bosectl_gui/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows_10_%7C_11-0078D4.svg)](#requirements)

**Control your Bose headphones from Windows. No Bose app, no cloud, no account.**

bosectl is a small Windows app that connects directly to your paired Bose
headphones over Bluetooth. It stays in the system tray, so your headphone
controls are always one click away.

<!-- Add a screenshot of the main window here, for example docs/images/screenshot.png -->

## Download

[**Download the latest installer**](https://github.com/kgr88/bosectl_gui/releases/latest)
(`bosectl_<version>_x64-setup.exe`).

## Features

- Change noise cancellation mode and strength.
- Adjust the bass, mid, and treble equalizer.
- Switch spatial audio modes.
- Edit and select listening profiles.
- Change device settings that your headphones report.
- See the battery level in the tray icon.
- Use the light, dark, or system color theme.

The app only shows the controls that your headphones support.

## Supported headphones

| Headphones | Status |
|------------|--------|
| QuietComfort Ultra Headphones (2nd Gen) | Tested |
| QuietComfort 35 / 35 II | Tested |

The app also recognizes other Bose headphones, earbuds, and speakers.
These devices are not tested yet. If you own one, please
[send a device report](https://github.com/kgr88/bosectl_gui/issues/new/choose).

## Requirements

- Windows 10 or 11.
- A Bluetooth adapter (built-in or USB).
- Bose headphones paired in Windows Bluetooth settings.

You do not need administrator rights, Python, or any other runtime.

## Install

1. Download the installer from the [latest release](https://github.com/kgr88/bosectl_gui/releases/latest).
2. Run the installer.
3. If Windows SmartScreen shows a warning, select **More info**, then **Run anyway**.
   The installer is not code-signed yet, so Windows shows this warning.
4. Start **bosectl** from the Start menu.

Optional: compare the installer hash with the `.sha256` file from the release:

```powershell
Get-FileHash .\bosectl_<version>_x64-setup.exe -Algorithm SHA256
```

## Use

1. Pair your headphones in Windows **Settings > Bluetooth & devices**.
2. Start bosectl. The app finds your paired headphones and connects.
3. Change settings in the main window.
4. Close the window to keep bosectl in the tray.
   Right-click the tray icon and select **Quit** to stop the app.

## Troubleshooting

- **The app does not find my headphones.** Make sure the headphones are paired
  in Windows and turned on. Disconnect them from your phone, then try again.
- **The connection drops.** Turn Bluetooth off and on in Windows, then reconnect
  in the app.
- **Something else is wrong.** Open the **Diagnostics** page in the app, then
  [open a bug report](https://github.com/kgr88/bosectl_gui/issues/new/choose)
  with that information.

## Uninstall

Open Windows **Settings > Apps > Installed apps**, find **bosectl**, and select
**Uninstall**.

## Privacy

bosectl talks only to your headphones. It does not connect to the internet,
collect data, or need an account.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup and checks.
Technical documentation is in [docs/](docs/).

## Credits and license

This project is a fork of [aaronsb/bosectl](https://github.com/aaronsb/bosectl).
It is released under the [MIT license](LICENSE).

bosectl is not affiliated with or endorsed by Bose Corporation.
"Bose" and "QuietComfort" are trademarks of Bose Corporation.
