# AWARIA - Aplikacja Wygodnego Alarmowanie o Remontach i Infrastrukturalnych Awariach

[English](README.md) | [Polski](README.pl.md)

<p align="center">
  <img height="600" alt="image" src="https://github.com/user-attachments/assets/eb73cb38-e146-4d16-a93d-af7c2662549c" />
</p>

A modern desktop (Tauri) and Android application providing real-time alerts for planned and emergency outages. **AWARIA** aggregates data from multiple utility providers into a centralized interface.

## Downloads

https://eremef.xyz/awaria

## Supported Sources

- **⚡ Power (Tauron)**: Planned maintenance and emergency power outages.
- **⚡ Power (Fortum)**: Planned and current power outages (Wrocław area).
- **💧 Water (MPWiK)**: Water failures and maintenance work (currently Wrocław area).

## Android app

<p align="center">
  <img height="600" alt="image" src="https://github.com/user-attachments/assets/ee991800-8960-4388-84e6-df3148b038ca" />
</p>

## Features

- **Multi-Source Logic**: Aggregates alerts from different utility providers (Power, Water, etc.).
- **Source Selection**: Customize which types of outages you want to see in the settings.
- **Smart Address Matching**: Highlights alerts affecting your specific address while keeping you informed about the surrounding area.
- **Premium Design**:
  - **Modern Interface**: Indigo-based "friendly" UI with vibrant source indicators (Rose/Sky).
  - **Collapsible categories**: Organized view of "Your Location" vs "Other Outages".
  - **Responsive Dark/Light mode**: Native transition support.
- **Android Widgets**:
  - **Individual Source Widgets**: Separate widgets for Power (Tauron, Fortum) and Water (MPWiK).
  - **Optimized Layout**: Compact 1x1 design showing alert counts for your specific street.
  - **One-tap refresh**: Tap the widget to trigger an immediate update.
  - **Shared configuration**: Settings sync automatically from the main app.
- **Privacy First**: No cloud accounts. Your location and settings stay on your device.

## Prerequisites

- Node.js (v18+)
- Rust (stable)
- Android Studio & SDK (for Android builds)
- Global Tauri CLI: `npm install -g @tauri-apps/cli`

## Setup

1. Install dependencies:

   ```bash
   npm install
   ```

## Development

### Desktop

Run the desktop app in development mode:

```bash
npm run dev
```

### Android

Run on a connected Android device or emulator:

```bash
npm run android
```

## Building

### Desktop app

Build the release bundle:

```bash
npm run build
```

### Android APK

Build the Android APK (unsigned/debug):

```bash
npm run android:build
```

The APK will be located at:
`src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`

## Architecture

- **Frontend**: Vanilla HTML/JS/CSS in `public/`. Indigo design system with custom HSL tokens.
- **Backend (Rust)**: `src-tauri/src/lib.rs` orchestrates asynchronous fetching from multiple APIs and converts them to a `UnifiedAlert` format.
- **Android Widgets**: Native implementation utilizing a `BaseWidgetProvider` with specific providers for each utility (`TauronWidgetProvider`, `MpwikWidgetProvider`). Includes a `WorkManager` background worker for periodic updates.

## Settings

Settings are stored in `settings.json` in the app's data directory:

- **Desktop**: `%APPDATA%\xyz.eremef.awaria\` (Windows)
- **Android**: `/data/user/0/xyz.eremef.awaria/files/`

## Troubleshooting

- **Widget shows "?"**: The settings haven't been configured yet. Open the main app and set your location.
- **EOF Errors**: Most likely a temporary race condition during settings sync. The app includes resilient logic to retry or fall back to defaults.
- **Missing Alerts**: Check if you have the specific outage category enabled in the settings.
