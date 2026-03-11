# Rustaria Browser Extension

Browser extension for sending downloads to the Rustaria download manager.

## Features

- Right-click context menu to download links, images, videos
- Automatic media detection on web pages
- HLS/DASH streaming manifest detection
- Cookie and header forwarding for authenticated downloads

## Build

```bash
cd extension
npm install
npm run build
```

## Install

### Chrome/Chromium

1. Open `chrome://extensions`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select the `extension` folder

### Firefox

1. Open `about:debugging`
2. Click "This Firefox"
3. Click "Load Temporary Add-on"
4. Select `extension/manifest.json`

## Native Messaging Host

The extension requires the native messaging host to be installed.
Run the following to install the manifest:

```bash
# Linux (Chrome)
mkdir -p ~/.config/google-chrome/NativeMessagingHosts/
cp native-host-manifest.json ~/.config/google-chrome/NativeMessagingHosts/ferrum_dl.json

# Linux (Firefox)
mkdir -p ~/.mozilla/native-messaging-hosts/
cp native-host-manifest.json ~/.mozilla/native-messaging-hosts/ferrum_dl.json
```

Edit the manifest to point to your rustaria binary.
