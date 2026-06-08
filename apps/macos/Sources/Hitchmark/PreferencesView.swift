//
//  PreferencesView.swift
//  Hitchmark
//
//  Settings/preferences window. All values persist via @AppStorage (UserDefaults).
//

import SwiftUI
import ServiceManagement

struct PreferencesView: View {
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
        .frame(width: 520, height: 420)
    }
}

// MARK: - General Tab

struct GeneralTab: View {
    @AppStorage("autoOpenLinks")   var autoOpenLinks   = true
    @AppStorage("launchAtLogin")   var launchAtLogin   = false
    @AppStorage("menuBarIconStyle") var menuBarIconStyle = 0

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            GroupBox(label: Text("Behavior")) {
                VStack(alignment: .leading, spacing: 12) {
                    Toggle("Automatically open links from hook:// URIs", isOn: $autoOpenLinks)
                    Toggle("Launch Hitchmark at login", isOn: $launchAtLogin)
                        .onChange(of: launchAtLogin) { newValue in
                            setLaunchAtLogin(newValue)
                        }
                }
                .padding()
            }

            GroupBox(label: Text("Appearance")) {
                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Menu bar icon style:")
                        Picker("", selection: $menuBarIconStyle) {
                            Text("Colored").tag(0)
                            Text("Monochrome").tag(1)
                        }
                        .pickerStyle(.segmented)
                        .frame(maxWidth: 200)
                    }
                }
                .padding()
            }

            Spacer()
        }
        .padding()
        .onAppear { syncLaunchAtLoginState() }
    }

    // MARK: - Launch at login (SMAppService, macOS 13+)

    private func setLaunchAtLogin(_ enabled: Bool) {
        do {
            if enabled {
                try SMAppService.mainApp.register()
            } else {
                try SMAppService.mainApp.unregister()
            }
        } catch {
            // Roll back the toggle if registration fails
            launchAtLogin = !enabled
            NSLog("Launch at login failed: \(error.localizedDescription)")
        }
    }

    /// Keep the toggle in sync with the actual SMAppService state on open
    private func syncLaunchAtLoginState() {
        let registered = SMAppService.mainApp.status == .enabled
        if launchAtLogin != registered {
            launchAtLogin = registered
        }
    }
}

// MARK: - Hotkey Tab

struct HotkeyTab: View {
    @AppStorage("useGlobalHotkey") var useGlobalHotkey = false
    @AppStorage("globalHotkey")    var globalHotkey    = "⌃⌥H"
    @State private var isRecordingHotkey = false
    @State private var accessibilityGranted = false

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            GroupBox(label: Text("Global Hotkey")) {
                VStack(alignment: .leading, spacing: 12) {
                    Toggle("Enable global hotkey", isOn: $useGlobalHotkey)
                        .onChange(of: useGlobalHotkey) { _ in
                            GlobalHotkeyManager.shared.configure()
                            if useGlobalHotkey { checkAccessibility() }
                        }

                    if useGlobalHotkey {
                        HStack {
                            Text("Hotkey:")
                            Spacer()
                            if isRecordingHotkey {
                                Text("Press key combination…")
                                    .foregroundColor(.blue)
                                // Invisible recorder view captures the key event
                                HotkeyRecorderView(hotkey: $globalHotkey, isRecording: $isRecordingHotkey)
                                    .frame(width: 1, height: 1)
                                    .onChange(of: globalHotkey) { _ in
                                        GlobalHotkeyManager.shared.configure()
                                    }
                                Button("Cancel") { isRecordingHotkey = false }
                                    .buttonStyle(.borderless)
                            } else {
                                Button(action: { isRecordingHotkey = true }) {
                                    Text(globalHotkey)
                                        .frame(minWidth: 80)
                                }
                                .buttonStyle(.bordered)
                                .help("Click then press your desired key combination")
                            }
                        }
                    }

                    Text("Default: ⌃⌥H (Ctrl+Option+H)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding()
            }

            if useGlobalHotkey && !accessibilityGranted {
                GroupBox {
                    HStack(spacing: 12) {
                        Image(systemName: "exclamationmark.triangle.fill")
                            .foregroundColor(.orange)
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Accessibility Permission Required")
                                .fontWeight(.medium)
                            Text("Global hotkeys require access to monitor keyboard input system-wide.")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        Spacer()
                        Button("Open Settings") {
                            GlobalHotkeyManager.openAccessibilitySettings()
                        }
                        .buttonStyle(.bordered)
                    }
                    .padding(4)
                }
            }

            Spacer()
        }
        .padding()
        .onAppear { checkAccessibility() }
    }

    private func checkAccessibility() {
        accessibilityGranted = GlobalHotkeyManager.accessibilityGranted()
    }
}

// MARK: - CLI Tab

struct CLITab: View {
    @AppStorage("cliPath")        var cliPath        = ""
    @AppStorage("serverUrl")      var serverUrl      = ""
    @AppStorage("autoStartServe") var autoStartServe = false
    @State private var detectedPath: String? = nil
    @State private var serverStatus: ServerStatus = .unknown

    enum ServerStatus { case unknown, reachable, unreachable }

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            GroupBox(label: Text("CLI Path")) {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Path to `hk` binary (leave blank to auto-detect):")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    HStack {
                        TextField("/usr/local/bin/hk", text: $cliPath)
                            .textFieldStyle(.roundedBorder)

                        Button(action: browseCLI) {
                            Image(systemName: "folder")
                        }
                        .help("Browse for hk binary")
                    }

                    if cliPath.isEmpty {
                        if let detected = detectedPath {
                            Label("Auto-detected: \(detected)", systemImage: "checkmark.circle.fill")
                                .font(.caption)
                                .foregroundColor(.green)
                        } else {
                            Label("Not found in /usr/local/bin, ~/.cargo/bin, /opt/homebrew/bin",
                                  systemImage: "exclamationmark.triangle")
                                .font(.caption)
                                .foregroundColor(.orange)
                        }
                    }
                }
                .padding()
            }

            GroupBox(label: Text("HTTP Server (hk serve)")) {
                VStack(alignment: .leading, spacing: 12) {
                    Text("When running `hk serve`, the app uses HTTP instead of subprocesses for faster link queries.")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    Toggle("Start hk serve automatically at login (via launchd)", isOn: $autoStartServe)
                        .onChange(of: autoStartServe) { newValue in
                            ServeAgent.setEnabled(newValue)
                        }

                    HStack {
                        TextField("http://127.0.0.1:2701", text: $serverUrl)
                            .textFieldStyle(.roundedBorder)

                        Button(action: probeServer) {
                            Image(systemName: "arrow.clockwise")
                        }
                        .help("Test connection")
                    }

                    switch serverStatus {
                    case .reachable:
                        Label("Server reachable", systemImage: "checkmark.circle.fill")
                            .font(.caption).foregroundColor(.green)
                    case .unreachable:
                        Label("Server not reachable", systemImage: "xmark.circle")
                            .font(.caption).foregroundColor(.red)
                    case .unknown:
                        EmptyView()
                    }
                }
                .padding()
            }

            Spacer()
        }
        .padding()
        .onAppear {
            detectCLI()
            if !serverUrl.isEmpty { probeServer() }
            autoStartServe = ServeAgent.isEnabled()
        }
    }

    private func browseCLI() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK, let url = panel.url {
            cliPath = url.path
        }
    }

    private func detectCLI() {
        detectedPath = HKBridge.locateHK()
    }

    private func probeServer() {
        guard let url = URL(string: "\(serverUrl)/health") else {
            serverStatus = .unreachable
            return
        }
        serverStatus = .unknown
        URLSession.shared.dataTask(with: URLRequest(url: url)) { _, response, _ in
            DispatchQueue.main.async {
                serverStatus = (response as? HTTPURLResponse)?.statusCode == 200
                    ? .reachable : .unreachable
            }
        }.resume()
    }
}

// MARK: - About Tab

struct AboutTab: View {
    private let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.1.0"

    var body: some View {
        VStack(alignment: .center, spacing: 20) {
            Image(systemName: "link.circle.fill")
                .font(.system(size: 64))
                .foregroundColor(.blue)

            Text("Hitchmark")
                .font(.title2)
                .fontWeight(.semibold)

            Text("v\(appVersion)")
                .font(.caption)
                .foregroundColor(.secondary)

            Text("Stable, addressable links to documents and paragraphs")
                .font(.body)
                .multilineTextAlignment(.center)

            Divider()

            VStack(alignment: .leading, spacing: 8) {
                Link("GitHub", destination: URL(string: "https://github.com/elw/hitchmark")!)
                Link("Documentation", destination: URL(string: "https://github.com/elw/hitchmark/blob/master/README.md")!)
                Link("Report Issue", destination: URL(string: "https://github.com/elw/hitchmark/issues")!)
            }
            .font(.caption)

            Spacer()

            Text("© 2026 Hitchmark Contributors · MIT License")
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
