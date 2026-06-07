//
//  PreferencesView.swift
//  Hookmarks
//
//  Settings/preferences window for configuring Hookmarks.
//

import SwiftUI

struct PreferencesView: View {
    @AppStorage("autoOpenLinks") var autoOpenLinks = true
    @AppStorage("useGlobalHotkey") var useGlobalHotkey = false
    @AppStorage("globalHotkey") var globalHotkey = "⌃⌥H"
    @AppStorage("cliPath") var cliPath = ""
    
    var body: some View {
        TabView {
            GeneralTab()
                .tabItem {
                    Label("General", systemImage: "gear")
                }
            
            HotkeyTab()
                .tabItem {
                    Label("Hotkeys", systemImage: "keyboard")
                }
            
            CLITab()
                .tabItem {
                    Label("CLI", systemImage: "terminal")
                }
            
            AboutTab()
                .tabItem {
                    Label("About", systemImage: "info.circle")
                }
        }
        .frame(width: 500, height: 400)
    }
}

// MARK: - General Tab

struct GeneralTab: View {
    @AppStorage("autoOpenLinks") var autoOpenLinks = true
    @AppStorage("launchAtLogin") var launchAtLogin = false
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            GroupBox(label: Text("Behavior")) {
                VStack(alignment: .leading, spacing: 12) {
                    Toggle("Automatically open links from hook:// URIs", isOn: $autoOpenLinks)
                    Toggle("Launch Hookmarks at login", isOn: $launchAtLogin)
                }
                .padding()
            }
            
            GroupBox(label: Text("Appearance")) {
                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Menu bar icon style:")
                        Picker("", selection: .constant(0)) {
                            Text("Colored").tag(0)
                            Text("Monochrome").tag(1)
                        }
                        .pickerStyle(.segmented)
                    }
                }
                .padding()
            }
            
            Spacer()
        }
        .padding()
    }
}

// MARK: - Hotkey Tab

struct HotkeyTab: View {
    @AppStorage("useGlobalHotkey") var useGlobalHotkey = false
    @State private var isRecordingHotkey = false
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            GroupBox(label: Text("Global Hotkey")) {
                VStack(alignment: .leading, spacing: 12) {
                    Toggle("Enable global hotkey", isOn: $useGlobalHotkey)
                    
                    if useGlobalHotkey {
                        HStack {
                            Text("Hotkey:")
                            Spacer()
                            if isRecordingHotkey {
                                Text("Press key combination...")
                                    .foregroundColor(.blue)
                            } else {
                                Button(action: { isRecordingHotkey = true }) {
                                    Text("⌃⌥H")
                                        .frame(minWidth: 80)
                                }
                                .buttonStyle(.bordered)
                            }
                        }
                    }
                    
                    Text("Suggested: ⌃⌥H (Ctrl+Option+H)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding()
            }
            
            Text("Global hotkeys are currently experimental and require accessibility permissions.")
                .font(.caption)
                .foregroundColor(.secondary)
            
            Spacer()
        }
        .padding()
    }
}

// MARK: - CLI Tab

struct CLITab: View {
    @AppStorage("cliPath") var cliPath = ""
    @State private var detectedPath: String? = nil
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            GroupBox(label: Text("CLI Configuration")) {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Hookmarks CLI path:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    HStack {
                        TextField("Path to hk", text: $cliPath)
                            .textFieldStyle(.roundedBorder)
                        
                        Button(action: browse) {
                            Image(systemName: "folder")
                        }
                    }
                    
                    if let detected = detectedPath {
                        Text("✓ Found at: \(detected)")
                            .font(.caption)
                            .foregroundColor(.green)
                    } else {
                        Text("ℹ️ Will search: /usr/local/bin, ~/.cargo/bin, /opt/homebrew/bin")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                .padding()
            }
            
            Spacer()
        }
        .padding()
        .onAppear { detectCLI() }
    }
    
    private func browse() {
        let panel = NSOpenPanel()
        panel.allowedFileTypes = ["hk", ""]
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        
        if panel.runModal() == .OK, let url = panel.url {
            cliPath = url.path
        }
    }
    
    private func detectCLI() {
        let paths = [
            "/usr/local/bin/hk",
            "\(NSHomeDirectory())/.cargo/bin/hk",
            "/opt/homebrew/bin/hk"
        ]
        
        detectedPath = paths.first { FileManager.default.fileExists(atPath: $0) }
    }
}

// MARK: - About Tab

struct AboutTab: View {
    var body: some View {
        VStack(alignment: .center, spacing: 20) {
            Image(systemName: "link.circle.fill")
                .font(.system(size: 64))
                .foregroundColor(.blue)
            
            Text("Hookmarks")
                .font(.title2)
                .fontWeight(.semibold)
            
            Text("v0.1.0")
                .font(.caption)
                .foregroundColor(.secondary)
            
            Text("Stable, addressable links to documents and paragraphs")
                .font(.body)
                .multilineTextAlignment(.center)
            
            Divider()
            
            VStack(alignment: .leading, spacing: 8) {
                Link("GitHub", destination: URL(string: "https://github.com/elw/not-hookmarks")!)
                Link("Documentation", destination: URL(string: "https://github.com/elw/not-hookmarks/blob/master/README.md")!)
                Link("Report Issue", destination: URL(string: "https://github.com/elw/not-hookmarks/issues")!)
            }
            .font(.caption)
            
            Spacer()
            
            Text("© 2024 Hookmarks Contributors\nMIT License")
                .font(.caption2)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }
}

#Preview {
    PreferencesView()
}
