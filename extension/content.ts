/**
 * Rustaria Browser Extension - Content Script
 *
 * Handles:
 * - Media detection on pages
 * - Video/audio element monitoring
 * - HLS/DASH manifest detection
 */

interface MediaInfo {
  url: string;
  type: "video" | "audio" | "hls" | "dash";
  title?: string;
}

/**
 * Find media elements on the page.
 */
function findMediaElements(): MediaInfo[] {
  const media: MediaInfo[] = [];

  // Find video elements
  document.querySelectorAll("video").forEach((video) => {
    if (video.src) {
      media.push({
        url: video.src,
        type: "video",
        title: document.title,
      });
    }

    // Check source elements
    video.querySelectorAll("source").forEach((source) => {
      if (source.src) {
        media.push({
          url: source.src,
          type: "video",
          title: document.title,
        });
      }
    });
  });

  // Find audio elements
  document.querySelectorAll("audio").forEach((audio) => {
    if (audio.src) {
      media.push({
        url: audio.src,
        type: "audio",
        title: document.title,
      });
    }
  });

  return media;
}

/**
 * Check for HLS/DASH manifests in network requests.
 */
function detectStreamingManifests(): void {
  // Override fetch to detect manifests
  const originalFetch = window.fetch;
  window.fetch = async function (...args) {
    const url = args[0] instanceof Request ? args[0].url : String(args[0]);

    if (url.includes(".m3u8") || url.includes(".mpd")) {
      chrome.runtime.sendMessage({
        type: "manifest_detected",
        url,
        pageUrl: window.location.href,
      });
    }

    return originalFetch.apply(this, args);
  };

  // Override XMLHttpRequest
  const originalOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method: string, url: string) {
    if (url.includes(".m3u8") || url.includes(".mpd")) {
      chrome.runtime.sendMessage({
        type: "manifest_detected",
        url,
        pageUrl: window.location.href,
      });
    }
    return originalOpen.apply(this, arguments as any);
  };
}

/**
 * Add download button to video elements.
 */
function addDownloadButtons(): void {
  document.querySelectorAll("video").forEach((video) => {
    // Skip if already processed
    if (video.dataset.ferrumProcessed) {
      return;
    }
    video.dataset.ferrumProcessed = "true";

    // Create download button (shown on hover)
    const button = document.createElement("button");
    button.textContent = "⬇️";
    button.title = "Download with Rustaria";
    button.style.cssText = `
      position: absolute;
      top: 10px;
      right: 10px;
      z-index: 9999;
      background: rgba(0, 0, 0, 0.7);
      color: white;
      border: none;
      border-radius: 4px;
      padding: 8px 12px;
      cursor: pointer;
      font-size: 16px;
      opacity: 0;
      transition: opacity 0.2s;
    `;

    button.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();

      const url = video.src || video.querySelector("source")?.src;
      if (url) {
        chrome.runtime.sendMessage({
          type: "download",
          url,
          filename: document.title + ".mp4",
        });
      }
    });

    // Wrap video in container if needed
    const parent = video.parentElement;
    if (parent) {
      parent.style.position = "relative";
      parent.appendChild(button);

      parent.addEventListener("mouseenter", () => {
        button.style.opacity = "1";
      });
      parent.addEventListener("mouseleave", () => {
        button.style.opacity = "0";
      });
    }
  });
}

// Initialize
detectStreamingManifests();

// Run on page load and mutations
const observer = new MutationObserver(() => {
  addDownloadButtons();
});

observer.observe(document.body, {
  childList: true,
  subtree: true,
});

// Initial run
addDownloadButtons();

// Handle messages from background script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "get_media") {
    sendResponse({ media: findMediaElements() });
  }
});

export {};
