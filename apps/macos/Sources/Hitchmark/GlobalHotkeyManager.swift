//
//  GlobalHotkeyManager.swift
//  Hitchmark
//
//  Registers a global (app-independent) key monitor that fires when the user
//  presses their configured hotkey from any application.
//
//  Requires Accessibility permission (System Settings → Privacy & Security → Accessibility).
//  The permission prompt is triggered the first time we install a monitor.
//

import Cocoa

extension Notification.Name {
    /// Posted on the main thread when the global hotkey fires.
    static let hitchmarkHotkeyActivated = Notification.Name("app.hitchmark.hotkeyActivated")
}

final class GlobalHotkeyManager {

    static let shared = GlobalHotkeyManager()
    private init() {}

    private var monitor: Any?

    // MARK: - Public API

    /// Call on launch and whenever the relevant UserDefaults keys change.
    func configure() {
        stop()

        let defaults = UserDefaults.standard
        guard defaults.bool(forKey: "useGlobalHotkey") else { return }

        let hotkeyStr = defaults.string(forKey: "globalHotkey") ?? "⌃⌥H"
        guard let (mods, keyChar) = parse(hotkeyStr) else {
            NSLog("GlobalHotkeyManager: cannot parse hotkey '\(hotkeyStr)'")
            return
        }

        // NSEvent.addGlobalMonitorForEvents requires Accessibility permission.
        // If it returns nil, the user hasn't granted access yet.
        monitor = NSEvent.addGlobalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self else { return }
            let eventMods = event.modifierFlags
                .intersection([.control, .option, .shift, .command])
            let eventChar = event.charactersIgnoringModifiers?.lowercased() ?? ""
            if eventMods == mods && eventChar == keyChar {
                self.fire()
            }
        }

        if monitor == nil {
            NSLog("GlobalHotkeyManager: monitor nil — Accessibility permission may be missing")
        } else {
            NSLog("GlobalHotkeyManager: registered '\(hotkeyStr)'")
        }
    }

    func stop() {
        if let m = monitor {
            NSEvent.removeMonitor(m)
            monitor = nil
        }
    }

    // MARK: - Helpers

    private func fire() {
        DispatchQueue.main.async {
            NotificationCenter.default.post(name: .hitchmarkHotkeyActivated, object: nil)
        }
    }

    /// Parse a display string like "⌃⌥H" or "⌘⇧K" → (modifiers, "h").
    /// Supported modifier symbols: ⌃ (ctrl) ⌥ (opt) ⇧ (shift) ⌘ (cmd)
    /// Modifiers may appear in any order.
    static func parse(_ str: String) -> (NSEvent.ModifierFlags, String)? {
        var mods: NSEvent.ModifierFlags = []
        var remaining = str

        let modMap: [(String, NSEvent.ModifierFlags)] = [
            ("⌃", .control), ("⌥", .option), ("⇧", .shift), ("⌘", .command),
        ]

        // Consume modifier symbols in any order until none remain at the front
        var progress = true
        while progress {
            progress = false
            for (symbol, flag) in modMap {
                if remaining.hasPrefix(symbol) {
                    mods.insert(flag)
                    remaining = String(remaining.dropFirst(symbol.count))
                    progress = true
                }
            }
        }

        guard let keyChar = remaining.first.map({ String($0).lowercased() }),
              !keyChar.isEmpty else { return nil }
        return (mods, keyChar)
    }

    private func parse(_ str: String) -> (NSEvent.ModifierFlags, String)? {
        GlobalHotkeyManager.parse(str)
    }

    // MARK: - Accessibility

    /// Returns true if Accessibility permission is granted.
    static func accessibilityGranted() -> Bool {
        AXIsProcessTrusted()
    }

    /// Opens the Accessibility pane in System Settings so the user can grant access.
    static func openAccessibilitySettings() {
        let url: URL
        if #available(macOS 13, *) {
            url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!
        } else {
            url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!
        }
        NSWorkspace.shared.open(url)
    }
}
