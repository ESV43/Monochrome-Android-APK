# Monochrome Android App

Android wrapper for [Monochrome](https://github.com/monochrome-music/monochrome), a privacy-respecting music streaming application.

## Features

- **Background playback** — Foreground Service keeps audio playing when the screen is off
- **Media controls** — Play/pause/skip in the notification shade, lock screen, and Bluetooth
- **Battery optimization bypass** — Requests exclusion from Android's battery killer on first launch
- **Downloads** — Saves tracks to `Downloads/MonochromeMusic/` with Android notification
- **Local files** — Select Music Folder works on Android (native folder picker)
- **OAuth Support** — Deep linking support for secure logins
- **Clipboard** — Copy to clipboard works natively
- **Bluetooth auto-pause** — Music pauses automatically when Bluetooth disconnects
- **Full UI** — Navigation bar visible, status bar visible with safe area padding
- **Back navigation** — Back button in header for album/artist/playlist navigation
- **Automation** — Built-in GitHub Actions for automatic APK generation

## Quick Start (Automatic)

The easiest way to get the APK is via GitHub Actions:
1. Fork this repository.
2. Go to the **Actions** tab in your fork.
3. Download the latest `Monochrome-debug-apk` artifact.

## Quick Start (Manual Build)

```bash
# 1. Clone Monochrome
git clone https://github.com/monochrome-music/monochrome.git
cd monochrome
git remote rename origin upstream

# 2. Clone this overlay
cd ..
git clone https://github.com/esv43/Monochrome-Android-APK

# 3. Install overlay into Monochrome
cd Monochrome-Android-APK
chmod +x install.sh
./install.sh ../monochrome

# 4. Build APK
cd ../monochrome
./build-android.sh
```

The APK will be at `Monochrome-debug.apk`.

## Updating

When Monochrome releases updates:

```bash
cd monochrome
./build-android.sh
```

The script automatically pulls the latest from upstream, applies patches, builds, and restores all files. **No manual work needed.**

## How It Works

The build script temporarily patches these upstream files during build:
- `index.html` — adds viewport-fit, script tag, brand name
- `package.json` — adds Capacitor dependencies

All patches are **reverted after build**. The upstream repo stays clean.

The Android-specific code lives entirely in:
- `android/` — Native Java code (foreground service, download bridge, etc.)
- `android/android-service.js` — JS bridge (media controls, downloads, CSS, back button)
- `capacitor.config.ts` — Capacitor configuration
- `build-android.sh` — Build automation

## Native Audio Engine (Experimental)

This version includes a native Rust audio engine for higher quality and better background performance.

### Build Requirements

1. **Rust** — `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`
2. **Cargo-NDK** — `cargo install cargo-ndk`
3. **Android NDK** — Ensure `ANDROID_NDK_HOME` is set.
4. **FFmpeg for Android** — The Rust engine links against native FFmpeg libraries.

## Architecture

```
Monochrome (upstream web app)
    │
    ├── Capacitor WebView (wraps the web app)
    │
    ├── android-service.js (injected at build time)
    │   ├── Download handler (monkey-patches <a download>)
    │   ├── Media controls (MutationObserver on document.title)
    │   ├── Native Audio redirection (sends URLs and EQ to Rust)
    │   ├── CSS injection (safe areas, layout fixes)
    │   ├── Back button (history.pushState hook)
    │   ├── Clipboard override (AndroidBridge)
    │   └── OAuth override (window.open → Chrome Custom Tab)
    │
    ├── Native Java
    │   ├── AudioForegroundService (MediaSession + Native Audio control)
    │   ├── AudioServicePlugin (Capacitor bridge)
    │   ├── NativeAudio (JNI wrapper for Rust)
    │   ├── DownloadBridge (MediaStore file saving)
    │   ├── LocalFilesBridge (Android folder picker)
    │   └── AndroidBridge (clipboard, browser)
    │
    └── Native Rust (Audio Engine)
        ├── CPAL (Audio Output)
        ├── FFmpeg (Decoding & Filtering)
        └── DSP (EQ & Volume)
```

## License

Same as [Monochrome](https://github.com/monochrome-music/monochrome/blob/main/license).
