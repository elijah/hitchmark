"use strict";
(() => {
  // src/content.ts
  document.addEventListener("hookmarks:copy-uri", () => {
    chrome.runtime.sendMessage(
      { type: "buildUri", payload: { url: window.location.href } },
      (res) => {
        if (res?.uri) {
          navigator.clipboard.writeText(res.uri).catch(() => void 0);
        }
      }
    );
  });
})();
//# sourceMappingURL=content.js.map
