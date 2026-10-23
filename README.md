# Grupr desktop

app.grupr.ai in its own window. Tauri 2, so the installer is a few megabytes and the window is the system webview (WebView2 on Windows, WKWebView on macOS).

What the shell adds over a browser tab:

- **One window that stays put.** Closing hides it; the tray icon brings it back; a second launch focuses the first.
- **Tray menu**: Open Grupr · Agents · Waiting on you · Quit.
- **Deep links**: `grupr://g/<room-id>`, `grupr://agents`, `grupr://agents/waiting` open the app on that page.
- **Links that leave Grupr open in your browser**, so the window never turns into a general browser. (`*.grupr.ai` and `github.com` stay inside so sign-ins and the GitHub App install complete.)
- **Remembers its size and position.**
- **Badge and notifications (0.2).** The web app reports how many things wait on you (approvals, requests, failing routines). The count shows on the tray tooltip, the window title and the macOS dock badge; the taskbar flashes when it grows; a native notification says so, at most once a minute.

The web page gets exactly two commands (`set_badge`, `notify`) and nothing else of Tauri; see `src-tauri/capabilities/default.json`.

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

## Auto-update

Since 0.3.0 the app checks this repository's latest release on launch (`latest.json`), installs a newer signed build and restarts. Builds are signed with the updater key held as the repository secrets `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (the private key file lives in the maintainer's `~/.tauri/`, never in the repo). The public key is in `src-tauri/tauri.conf.json`. The update endpoint is only reachable while the repository is public.

## Code signing

Both are wired in the workflow and switch on when their secrets exist.

- **Windows** (Azure Trusted Signing): secrets `AZURE_SIGN_ENDPOINT` (e.g. `https://eus.codesigning.azure.net`), `AZURE_SIGN_ACCOUNT`, `AZURE_SIGN_PROFILE`, and a service principal with the *Trusted Signing Certificate Profile Signer* role: `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`. The workflow installs `trusted-signing-cli` and signs the NSIS and MSI installers.
- **macOS** (Developer ID + notarization): `APPLE_CERTIFICATE` (the .p12, base64), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (`Developer ID Application: Grupr Inc (TEAMID)`), `APPLE_ID`, `APPLE_PASSWORD` (an app-specific password), `APPLE_TEAM_ID`. tauri-action signs and notarizes the .app and .dmg when these are present.
