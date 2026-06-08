//
//  ServeAgent.swift
//  Hitchmark
//
//  Manages the launchd LaunchAgent that auto-starts `hk serve` at login.
//  The plist is written dynamically so we can embed the correct hk path.
//

import Foundation

enum ServeAgent {

    private static let label = "app.hitchmark.serve"
    private static var plistURL: URL {
        let agents = FileManager.default
            .homeDirectoryForCurrentUser
            .appendingPathComponent("Library/LaunchAgents")
        return agents.appendingPathComponent("\(label).plist")
    }

    // MARK: - Public API

    static func isEnabled() -> Bool {
        FileManager.default.fileExists(atPath: plistURL.path)
    }

    static func setEnabled(_ enabled: Bool) {
        if enabled {
            install()
        } else {
            uninstall()
        }
    }

    // MARK: - Install

    private static func install() {
        guard let hkPath = HKBridge.locateHK() else {
            NSLog("ServeAgent: hk binary not found, cannot install launchd agent")
            return
        }

        let plistContent = makePlist(hkPath: hkPath)
        do {
            let dir = plistURL.deletingLastPathComponent()
            try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
            try plistContent.write(to: plistURL, atomically: true, encoding: .utf8)
            launchctl("unload", plistURL.path)   // remove stale copy if present
            launchctl("load", "-w", plistURL.path)
            NSLog("ServeAgent: installed and loaded \(plistURL.path)")
        } catch {
            NSLog("ServeAgent: failed to write plist: \(error)")
        }
    }

    // MARK: - Uninstall

    private static func uninstall() {
        launchctl("unload", plistURL.path)
        try? FileManager.default.removeItem(at: plistURL)
        NSLog("ServeAgent: unloaded and removed \(plistURL.path)")
    }

    // MARK: - Helpers

    @discardableResult
    private static func launchctl(_ args: String...) -> Int32 {
        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: "/bin/launchctl")
        proc.arguments = args
        proc.standardOutput = FileHandle.nullDevice
        proc.standardError  = FileHandle.nullDevice
        try? proc.run()
        proc.waitUntilExit()
        return proc.terminationStatus
    }

    private static func makePlist(hkPath: String) -> String {
        """
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
          "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
          <key>Label</key>
          <string>\(label)</string>
          <key>ProgramArguments</key>
          <array>
            <string>\(hkPath)</string>
            <string>serve</string>
          </array>
          <key>RunAtLoad</key>
          <true/>
          <key>KeepAlive</key>
          <true/>
          <key>ThrottleInterval</key>
          <integer>10</integer>
          <key>StandardOutPath</key>
          <string>/tmp/hitchmark-serve.log</string>
          <key>StandardErrorPath</key>
          <string>/tmp/hitchmark-serve.err</string>
        </dict>
        </plist>
        """
    }
}
