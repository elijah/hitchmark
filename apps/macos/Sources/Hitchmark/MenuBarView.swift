//
//  MenuBarView.swift
//  Hitchmark
//
//  Main menu bar interface for quick access to hookmark features.
//

import SwiftUI

struct MenuBarView: View {
    @State private var selectedTab = 0
    @State private var isLoading = false
    @State private var errorMessage: String? = nil
    
    var body: some View {
        VStack(spacing: 0) {
            // Tab picker
            Picker("", selection: $selectedTab) {
                Label("Link", systemImage: "link").tag(0)
                Label("List", systemImage: "list.bullet").tag(1)
                Label("Finder", systemImage: "folder").tag(2)
            }
            .pickerStyle(.segmented)
            .padding()
            
            Divider()
            
            // Tab content
            Group {
                if selectedTab == 0 {
                    LinkTabView()
                } else if selectedTab == 1 {
                    ListTabView()
                } else {
                    FinderTabView()
                }
            }
            .padding()
            
            // Error display
            if let error = errorMessage {
                Divider()
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
                    .padding()
            }
            
            Divider()
            
            // Footer
            HStack {
                Button(action: openPreferences) {
                    Image(systemName: "gear")
                        .font(.system(size: 12))
                }
                .help("Preferences")
                
                Spacer()
                
                Text("v0.1.0")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        }
        .frame(height: 300)
    }
    
    private func openPreferences() {
        NSApp.keyWindow?.close()
        
        // Try to find and focus existing preferences window
        for window in NSApplication.shared.windows {
            if window.title.contains("Preferences") || window.title.contains("Settings") {
                window.makeKeyAndOrderFront(nil)
                return
            }
        }
        
        // Fallback: open Preferences via menu
        NSApp.sendAction(Selector(("showPreferencesWindow:")), to: nil, from: nil)
    }
}

// MARK: - Tab: Link

struct LinkTabView: View {
    @State private var uriA = ""
    @State private var uriB = ""
    @State private var note = ""
    @State private var isLoading = false
    
    var body: some View {
        VStack(spacing: 12) {
            Text("Create Link")
                .font(.headline)
            
            TextField("URI or path A", text: $uriA)
                .textFieldStyle(.roundedBorder)
            
            TextField("URI or path B", text: $uriB)
                .textFieldStyle(.roundedBorder)
            
            TextField("Note (optional)", text: $note)
                .textFieldStyle(.roundedBorder)
            
            Button(action: createLink) {
                if isLoading {
                    ProgressView()
                        .scaleEffect(0.8)
                } else {
                    Text("Create Link")
                }
            }
            .disabled(uriA.isEmpty || uriB.isEmpty || isLoading)
        }
    }
    
    private func createLink() {
        isLoading = true
        HKBridge.link(uriA: uriA, uriB: uriB, note: note.isEmpty ? nil : note) { result in
            isLoading = false
            switch result {
            case .success:
                uriA = ""
                uriB = ""
                note = ""
            case .failure(let error):
                NSLog("Link failed: \(error)")
            }
        }
    }
}

// MARK: - Tab: List

struct ListTabView: View {
    @State private var uri = ""
    @State private var links: String = ""
    @State private var isLoading = false
    
    var body: some View {
        VStack(spacing: 12) {
            Text("Show Links")
                .font(.headline)
            
            TextField("URI or path", text: $uri)
                .textFieldStyle(.roundedBorder)
            
            Button(action: queryLinks) {
                if isLoading {
                    ProgressView()
                        .scaleEffect(0.8)
                } else {
                    Text("Query Links")
                }
            }
            .disabled(uri.isEmpty || isLoading)
            
            if !links.isEmpty {
                ScrollView {
                    Text(links)
                        .font(.system(.body, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .textSelection(.enabled)
                }
                .frame(maxHeight: .infinity)
                .border(Color.gray.opacity(0.3))
            }
        }
    }
    
    private func queryLinks() {
        isLoading = true
        HKBridge.list(uri: uri) { result in
            isLoading = false
            switch result {
            case .success(let output):
                links = output
            case .failure(let error):
                links = "Error: \(error.localizedDescription)"
            }
        }
    }
}

// MARK: - Tab: Finder

struct FinderTabView: View {
    @State private var selectedPath: String? = nil
    @State private var generatedURI: String? = nil
    @State private var isLoading = false
    
    var body: some View {
        VStack(spacing: 12) {
            Text("Link for Finder Selection")
                .font(.headline)
            
            Button(action: getFinderSelection) {
                if isLoading {
                    ProgressView()
                        .scaleEffect(0.8)
                } else {
                    Text("Get Selected File")
                }
            }
            .disabled(isLoading)
            
            if let path = selectedPath {
                Text("Selected: \(path)")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .lineLimit(2)
            }
            
            if let uri = generatedURI {
                VStack(spacing: 8) {
                    Text("Hook URI:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    HStack {
                        Text(uri)
                            .font(.system(.caption, design: .monospaced))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .lineLimit(3)
                        
                        Button(action: copyToClipboard) {
                            Image(systemName: "doc.on.doc")
                                .font(.system(size: 12))
                        }
                        .help("Copy to clipboard")
                    }
                    .padding(8)
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(4)
                }
            }
        }
    }
    
    private func getFinderSelection() {
        isLoading = true
        FinderBridge.getSelectedFile { path in
            selectedPath = path
            if let path = path {
                HKBridge.fileToURI(path) { result in
                    isLoading = false
                    switch result {
                    case .success(let uri):
                        generatedURI = uri
                    case .failure(let error):
                        NSLog("Error: \(error)")
                    }
                }
            } else {
                isLoading = false
            }
        }
    }
    
    private func copyToClipboard() {
        if let uri = generatedURI {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(uri, forType: .string)
        }
    }
}

#Preview {
    MenuBarView()
}
