//
//  ServicesHandler.swift
//  Hookmarks
//
//  Handles the 5 macOS System Services registered in Info.plist.
//  Registered as NSApp.servicesProvider in AppDelegate.
//
//  Services appear in:
//    • Right-click → Services → Hookmarks/…  (Finder, text editors, etc.)
//    • Application menu → Services → Hookmarks/…
//

import Cocoa

@objc class ServicesHandler: NSObject {

    // MARK: - Service 1: Copy hook:// URI for selected files

    /// Receives file selection from Finder (or any app that sends file URLs).
    /// Converts the first selected file to a hook:// URI and copies it to the clipboard.
    @objc func hookmarksCopyURI(
        _ pboard: NSPasteboard,
        userData: String?,
        error: AutoreleasingUnsafeMutablePointer<NSString?>?
    ) {
        guard let urls = fileURLs(from: pboard), let first = urls.first else {
            error?.pointee = "No file found in selection." as NSString
            return
        }

        HKBridge.fileToURI(first.path) { result in
            DispatchQueue.main.async {
                switch result {
                case .success(let uri):
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(uri, forType: .string)
                    NSSound(named: NSSound.Name("Tink"))?.play()
                case .failure(let err):
                    self.showError("Copy URI Failed", err.localizedDescription)
                }
            }
        }
    }

    // MARK: - Service 2: Link two selected files

    /// Receives 2+ files from Finder.
    /// Converts both to hook:// URIs and creates a bidirectional link between them.
    @objc func hookmarksLinkFiles(
        _ pboard: NSPasteboard,
        userData: String?,
        error: AutoreleasingUnsafeMutablePointer<NSString?>?
    ) {
        guard let urls = fileURLs(from: pboard), urls.count >= 2 else {
            error?.pointee = "Select exactly two files to link." as NSString
            DispatchQueue.main.async {
                self.showError(
                    "Link Files",
                    "Select exactly two files in Finder to create a Hookmarks link between them."
                )
            }
            return
        }

        let (urlA, urlB) = (urls[0], urls[1])

        HKBridge.fileToURI(urlA.path) { resultA in
            guard case .success(let uriA) = resultA else {
                DispatchQueue.main.async {
                    self.showError("Link Files", "Could not resolve URI for \(urlA.lastPathComponent)")
                }
                return
            }
            HKBridge.fileToURI(urlB.path) { resultB in
                guard case .success(let uriB) = resultB else {
                    DispatchQueue.main.async {
                        self.showError("Link Files", "Could not resolve URI for \(urlB.lastPathComponent)")
                    }
                    return
                }
                HKBridge.link(uriA: uriA, uriB: uriB) { linkResult in
                    DispatchQueue.main.async {
                        switch linkResult {
                        case .success:
                            self.showInfo(
                                "Files Linked ✓",
                                "\(urlA.lastPathComponent)  ↔  \(urlB.lastPathComponent)"
                            )
                        case .failure(let err):
                            self.showError("Link Failed", err.localizedDescription)
                        }
                    }
                }
            }
        }
    }

    // MARK: - Service 3: Show links for selected file

    /// Receives a file from Finder.
    /// Displays an alert listing all documents linked to that file.
    @objc func hookmarksShowLinks(
        _ pboard: NSPasteboard,
        userData: String?,
        error: AutoreleasingUnsafeMutablePointer<NSString?>?
    ) {
        guard let urls = fileURLs(from: pboard), let first = urls.first else {
            error?.pointee = "No file found in selection." as NSString
            return
        }

        HKBridge.fileToURI(first.path) { result in
            guard case .success(let uri) = result else {
                DispatchQueue.main.async {
                    self.showError("Show Links", "Could not resolve URI for \(first.lastPathComponent)")
                }
                return
            }
            HKBridge.list(uri: uri) { listResult in
                DispatchQueue.main.async {
                    switch listResult {
                    case .success(let json):
                        self.presentLinksAlert(filename: first.lastPathComponent, uri: uri, json: json)
                    case .failure(let err):
                        self.showError("Show Links", err.localizedDescription)
                    }
                }
            }
        }
    }

    // MARK: - Service 4: Open hook:// URI from text selection

    /// Receives selected text from any app (text editor, browser, etc.).
    /// Extracts the first hook:// URI found and opens the linked document.
    @objc func hookmarksOpenURI(
        _ pboard: NSPasteboard,
        userData: String?,
        error: AutoreleasingUnsafeMutablePointer<NSString?>?
    ) {
        guard let text = pboard.string(forType: .string) else {
            error?.pointee = "No text selected." as NSString
            return
        }

        let pattern = #"hook://[^\s\"<>]+"#
        guard let range = text.range(of: pattern, options: .regularExpression) else {
            error?.pointee = "No hook:// URI found in selected text." as NSString
            DispatchQueue.main.async {
                self.showError("Open URI", "No hook:// URI found in selected text.\n\nURI must start with hook://")
            }
            return
        }

        let uri = String(text[range])
        HKBridge.open(uri: uri) { result in
            if case .failure(let err) = result {
                DispatchQueue.main.async {
                    self.showError("Open URI Failed", err.localizedDescription)
                }
            }
        }
    }

    // MARK: - Service 5: Convert file path text → hook:// URI (in/out replacement)

    /// Receives selected text from any text editor (a file path).
    /// Replaces the selection with the corresponding hook:// URI.
    ///
    /// Example: select "/Users/alice/notes/ideas.md" in any editor,
    /// invoke this service, and the selection becomes "hook://file/..."
    @objc func hookmarksConvertPath(
        _ pboard: NSPasteboard,
        userData: String?,
        error: AutoreleasingUnsafeMutablePointer<NSString?>?
    ) {
        guard let raw = pboard.string(forType: .string)?
            .trimmingCharacters(in: .whitespacesAndNewlines),
              !raw.isEmpty else {
            error?.pointee = "No text selected." as NSString
            return
        }

        let expanded = (raw as NSString).expandingTildeInPath
        guard FileManager.default.fileExists(atPath: expanded) else {
            error?.pointee = "File not found: \(raw)" as NSString
            DispatchQueue.main.async {
                self.showError("Convert Path", "File not found:\n\(raw)\n\nSelect a valid file path.")
            }
            return
        }

        // NSServices in/out: write back to the pasteboard synchronously
        let sem = DispatchSemaphore(value: 0)
        var resolved: String?

        HKBridge.fileToURI(expanded) { result in
            if case .success(let uri) = result { resolved = uri }
            sem.signal()
        }
        sem.wait()

        guard let uri = resolved else {
            error?.pointee = "Could not get URI for \(raw)" as NSString
            return
        }
        pboard.clearContents()
        pboard.setString(uri, forType: .string)
    }

    // MARK: - Private helpers

    private func fileURLs(from pboard: NSPasteboard) -> [URL]? {
        let urls = pboard.readObjects(
            forClasses: [NSURL.self],
            options: [.urlReadingFileURLsOnly: true]
        ) as? [URL]
        return urls?.isEmpty == false ? urls : nil
    }

    private func showError(_ title: String, _ message: String) {
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.alertStyle = .warning
        NSApp.activate(ignoringOtherApps: true)
        alert.runModal()
    }

    private func showInfo(_ title: String, _ message: String) {
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.alertStyle = .informational
        NSApp.activate(ignoringOtherApps: true)
        alert.runModal()
    }

    private func presentLinksAlert(filename: String, uri: String, json: String) {
        struct LinkRecord: Codable {
            let uri_a: String
            let uri_b: String
            let note: String?
        }

        let data = json.data(using: .utf8) ?? Data()
        let links = (try? JSONDecoder().decode([LinkRecord].self, from: data)) ?? []

        let alert = NSAlert()
        alert.messageText = "Links for \(filename)"

        if links.isEmpty {
            alert.informativeText = "No links found for this file.\n\nUse \u{201C}Link Selected Files\u{201D} to create one."
        } else {
            let lines = links.enumerated().map { (i, link) -> String in
                let other = link.uri_a == uri ? link.uri_b : link.uri_a
                let noteStr = link.note.map { "  — \($0)" } ?? ""
                // Trim the hook://file/ prefix for readability, decode base64 path
                let display = decodedPath(from: other) ?? other
                return "\(i + 1). \(display)\(noteStr)"
            }
            alert.informativeText = lines.joined(separator: "\n")
        }

        // "Copy URI" as the default button so the URI is easy to grab
        alert.addButton(withTitle: "Copy URI")
        alert.addButton(withTitle: "OK")

        NSApp.activate(ignoringOtherApps: true)
        if alert.runModal() == .alertFirstButtonReturn {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(uri, forType: .string)
        }
    }

    /// Decode a hook://file/<base64url> URI back to a readable path for display.
    private func decodedPath(from uri: String) -> String? {
        guard uri.hasPrefix("hook://file/") else { return nil }
        var encoded = String(uri.dropFirst("hook://file/".count))
        // Strip fragment if present
        if let hashIdx = encoded.firstIndex(of: "#") {
            encoded = String(encoded[..<hashIdx])
        }
        // Restore standard base64 padding
        let padded = encoded
            .replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/")
        let remainder = padded.count % 4
        let padStr = remainder > 0 ? String(repeating: "=", count: 4 - remainder) : ""
        guard let data = Data(base64Encoded: padded + padStr),
              let path = String(data: data, encoding: .utf8) else { return nil }
        // Abbreviate home directory
        return path.replacingOccurrences(of: NSHomeDirectory(), with: "~")
    }
}
