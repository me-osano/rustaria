/**
 * Rustaria Browser Extension - Background Service Worker
 *
 * Handles:
 * - Native messaging communication with rustaria
 * - Download interception
 * - Context menu integration
 * - Media sniffing coordination
 */

const NATIVE_HOST_NAME = "ferrum_dl";

interface NativeResponse {
  success: boolean;
  data?: unknown;
  error?: string;
}

interface DownloadRequest {
  type: "add_download";
  url: string;
  filename?: string;
  headers?: Array<{ name: string; value: string }>;
  cookies?: string;
  referer?: string;
}

// Native messaging port
let port: chrome.runtime.Port | null = null;

/**
 * Connect to the native messaging host.
 */
function connectNative(): chrome.runtime.Port {
  if (port) {
    return port;
  }

  port = chrome.runtime.connectNative(NATIVE_HOST_NAME);

  port.onMessage.addListener((response: NativeResponse) => {
    console.log("Native response:", response);
  });

  port.onDisconnect.addListener(() => {
    console.log("Native host disconnected:", chrome.runtime.lastError?.message);
    port = null;
  });

  return port;
}

/**
 * Send a message to the native host.
 */
async function sendNativeMessage(message: unknown): Promise<NativeResponse> {
  return new Promise((resolve) => {
    chrome.runtime.sendNativeMessage(
      NATIVE_HOST_NAME,
      message,
      (response: NativeResponse) => {
        if (chrome.runtime.lastError) {
          resolve({
            success: false,
            error: chrome.runtime.lastError.message,
          });
        } else {
          resolve(response);
        }
      }
    );
  });
}

/**
 * Send a download to rustaria.
 */
async function sendDownload(
  url: string,
  filename?: string,
  referer?: string
): Promise<boolean> {
  const request: DownloadRequest = {
    type: "add_download",
    url,
    filename,
    referer,
  };

  // Get cookies for the URL
  try {
    const urlObj = new URL(url);
    const cookies = await chrome.cookies.getAll({ domain: urlObj.hostname });
    request.cookies = cookies.map((c) => `${c.name}=${c.value}`).join("; ");
  } catch (e) {
    console.warn("Failed to get cookies:", e);
  }

  const response = await sendNativeMessage(request);
  return response.success;
}

/**
 * Create context menu items.
 */
function createContextMenus(): void {
  chrome.contextMenus.create({
    id: "rustaria-link",
    title: "Download with Rustaria",
    contexts: ["link"],
  });

  chrome.contextMenus.create({
    id: "rustaria-media",
    title: "Download media with Rustaria",
    contexts: ["video", "audio"],
  });

  chrome.contextMenus.create({
    id: "rustaria-image",
    title: "Download image with Rustaria",
    contexts: ["image"],
  });
}

// Context menu click handler
chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  let url: string | undefined;

  switch (info.menuItemId) {
    case "rustaria-link":
      url = info.linkUrl;
      break;
    case "rustaria-media":
      url = info.srcUrl;
      break;
    case "rustaria-image":
      url = info.srcUrl;
      break;
  }

  if (url) {
    const success = await sendDownload(url, undefined, tab?.url);
    if (success) {
      console.log("Download sent:", url);
    } else {
      console.error("Failed to send download:", url);
    }
  }
});

// Download interception (optional - intercept browser downloads)
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  // Check if we should intercept this download
  // For now, let browser handle small files
  if (downloadItem.fileSize && downloadItem.fileSize < 10 * 1024 * 1024) {
    return;
  }

  // Cancel browser download and send to rustaria
  // chrome.downloads.cancel(downloadItem.id);
  // await sendDownload(downloadItem.url, downloadItem.filename);
});

// Ping native host on startup
chrome.runtime.onInstalled.addListener(async () => {
  createContextMenus();

  const response = await sendNativeMessage({ type: "ping" });
  if (response.success) {
    console.log("Rustaria native host is available");
  } else {
    console.warn("Rustaria native host not available:", response.error);
  }
});

// Handle messages from content scripts
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "download") {
    sendDownload(message.url, message.filename, sender.tab?.url).then(
      (success) => {
        sendResponse({ success });
      }
    );
    return true; // Async response
  }
});

export {};
