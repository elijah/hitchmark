/**
 * Hitchmark OneNote Add-in — ribbon command functions.
 * These run as "function commands" (no task pane, immediate execution).
 */

import { HKAddInBridge } from "./bridge";

const bridge = new HKAddInBridge();

/** Copy the current page's hook:// URI — invoked from the ribbon button */
export async function copyCurrentPageUri(event: Office.AddinCommands.Event): Promise<void> {
  try {
    await OneNote.run(async (context) => {
      const page = context.application.getActivePage();
      page.load("webUrl");
      await context.sync();

      const uri = HKAddInBridge.buildPageUri(page.webUrl);
      await navigator.clipboard.writeText(uri);
    });
  } catch (e) {
    console.error("Hitchmark copyCurrentPageUri failed:", e);
  } finally {
    event.completed();
  }
}

// Register function commands with Office
(Office as unknown as { actions: { associate: (name: string, fn: unknown) => void } })
  .actions?.associate("copyCurrentPageUri", copyCurrentPageUri);
