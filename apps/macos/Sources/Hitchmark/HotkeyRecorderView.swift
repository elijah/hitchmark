//
//  HotkeyRecorderView.swift
//  Hitchmark
//
//  A SwiftUI-compatible view that listens for a key-down event and records it
//  as a modifier+key display string (e.g. "⌃⌥H").
//
//  Usage:
//    HotkeyRecorderView(hotkey: $globalHotkey, isRecording: $isRecordingHotkey)
//

import SwiftUI
import AppKit

struct HotkeyRecorderView: NSViewRepresentable {
    @Binding var hotkey: String
    @Binding var isRecording: Bool

    func makeNSView(context: Context) -> HotkeyField {
        let field = HotkeyField()
        field.onHotkeyRecorded = { recorded in
            DispatchQueue.main.async {
                if let recorded {
                    hotkey = recorded
                }
                isRecording = false
            }
        }
        return field
    }

    func updateNSView(_ nsView: HotkeyField, context: Context) {
        if isRecording {
            nsView.window?.makeFirstResponder(nsView)
        }
    }
}

// MARK: -

class HotkeyField: NSView {
    /// Called with the recorded string, or nil if the user cancelled (Escape).
    var onHotkeyRecorded: ((String?) -> Void)?

    override var acceptsFirstResponder: Bool { true }
    override var isFlipped: Bool { false }

    override func keyDown(with event: NSEvent) {
        let chars = event.charactersIgnoringModifiers ?? ""

        // Escape → cancel
        if chars == "\u{1b}" {
            onHotkeyRecorded?(nil)
            return
        }
        // Ignore bare modifier-only presses (Tab, Return, Delete, lone modifier)
        if chars.isEmpty || chars == "\t" || chars == "\r" || chars == "\u{7f}" {
            return
        }

        let mods = event.modifierFlags.intersection([.control, .option, .shift, .command])
        // Require at least one modifier so single letters don't steal input
        guard !mods.isEmpty else { return }

        var result = ""
        if mods.contains(.control) { result += "⌃" }
        if mods.contains(.option)  { result += "⌥" }
        if mods.contains(.shift)   { result += "⇧" }
        if mods.contains(.command) { result += "⌘" }
        result += chars.uppercased()

        onHotkeyRecorded?(result)
    }

    // Accept mouse clicks so we can become first responder by clicking the button
    override func mouseDown(with event: NSEvent) {
        window?.makeFirstResponder(self)
    }
}
