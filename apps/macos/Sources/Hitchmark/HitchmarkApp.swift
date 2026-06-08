//
//  HitchmarkApp.swift
//  Hitchmark
//
//  Main entry point for the Hitchmark macOS application.
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
