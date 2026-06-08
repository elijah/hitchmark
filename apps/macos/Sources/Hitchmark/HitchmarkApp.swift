//
//  HitchmarkApp.swift
//  Hookmarks
//
//  Main entry point for the Hookmarks macOS application.
//

import SwiftUI

@main
struct HitchmarkApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    var body: some Scene {
        MenuBarExtra("Hitchmark", systemImage: "link.circle.fill") {
            MenuBarView()
                .frame(minWidth: 300)
        }
        .menuBarExtraStyle(.window)
        
        Settings {
            PreferencesView()
        }
    }
}
