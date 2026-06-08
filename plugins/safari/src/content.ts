/**
 * Content script — injected into every page.
 * Provides hook:// URI copy shortcuts and keyboard trigger.
 */

// Listen for keyboard shortcut (Shift+Cmd+H on mac, triggered via popup)
document.addEventListener("hookmarks:copy-uri", () => {
  chrome.runtime.sendMessage(
    { type: "buildUri", payload: { url: window.location.href } },
    (res: { uri: string }) => {
      if (res?.uri) {
        navigator.clipboard.writeText(res.uri).catch(() => undefined);
      }
    },
  );
});
