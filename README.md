# Grupr desktop

app.grupr.ai in its own window. Tauri 2, so the installer is a few megabytes and the window is the system webview (WebView2 on Windows, WKWebView on macOS).

What the shell adds over a browser tab:

- **One window that stays put.** Closing hides it; the tray icon brings it back; a second launch focuses the first.
- **Tray menu**: Open Grupr · Agents · Waiting on you · Quit.
- **Deep links**: `grupr://g/<room-id>`, `grupr://agents`, `grupr://agents/waiting` open the app on that page.
- **Links that leave Grupr open in your browser**, so the window never turns into a general browser. (`*.grupr.ai` and `github.com` stay inside so sign-ins and the GitHub App install complete.)
- **Remembers its size and position.**

The web page runs exactly as it does in a browser: it is given no Tauri IPC. Notifications and an unread badge come in a later version through a small declared bridge.

## Install

Download the latest installer from the [releases](https://github.com/grupr-ai/grupr-desktop/releases) (Windows `.exe`/`.msi`, macOS `.dmg`). Every push to `main` also leaves installers as workflow artifacts under Actions.

Until the builds are signed, Windows SmartScreen says "Windows protected your PC" (More info → Run anyway) and macOS says the app is from an unidentified developer (right-click → Open once).

## Build locally

Needs Node 20+ and a Rust toolchain (`rustup`), plus the Tauri prerequisites for your OS (WebView2 is already on Windows 11).

```bash
npm ci
npm run dev      # runs the shell against https://app.grupr.ai
npm run build    # installers under src-tauri/target/release/bundle
```

`GRUPR_APP_URL=http://localhost:3002 npm run dev` points the window at a local web app.

## Releasing

Bump `version` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json`, commit, tag `vX.Y.Z`, push the tag. The workflow drafts a GitHub release with both installers; publish it from the Releases page.

## Signing and auto-update (not yet set up)

- **Windows**: an Authenticode certificate (an EV or OV cert, or Azure Trusted Signing). Set `bundle.windows.certificateThumbprint` or the `signCommand`, and add the secrets to the workflow.
- **macOS**: an Apple Developer ID Application certificate plus notarization (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` as workflow secrets; tauri-action picks them up).
- **Auto-update**: `npm run tauri signer generate -- -w ~/.tauri/grupr.key` once, keep the private key and its password out of the repository, add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as workflow secrets, put the public key in `tauri.conf.json` under `plugins.updater.pubkey`, and add `tauri-plugin-updater`. The release workflow then publishes `latest.json` next to the installers.
