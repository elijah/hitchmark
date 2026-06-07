//
//  HookmarksApp.swift
//  Hookmarks
//
//  Main entry point for the Hookmarks macOS application.
//

import SwiftUI

@main
struct HookmarksApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    var body: some Scene {
        MenuBarExtra("Hookmarks", systemImage: "link.circle.fill") {
            MenuBarView()
                .frame(minWidth: 300)
        }
        .menuBarExtraStyle(.window)
        
        Settings {
            PreferencesView()
        }
    }
}
