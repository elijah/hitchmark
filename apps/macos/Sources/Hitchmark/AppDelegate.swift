//
//  AppDelegate.swift
//  Hookmarks
//
//  Handles application lifecycle and hook:// URL scheme.
//

import Cocoa
import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {

    private let servicesHandler = ServicesHandler()

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSLog("Hitchmark launched")
        // Register system services and refresh the Services menu cache
        NSApp.servicesProvider = servicesHandler
        NSUpdateDynamicServices()
    }
    
    // Handle hook:// URIs opened from Safari, Finder, or command line
    func application(
        _ application: NSApplication,
        open urls: [URL]
    ) {
        for url in urls {
            handleHookmarkURI(url)
        }
    }
    
    private func handleHookmarkURI(_ url: URL) {
        guard url.scheme == "hook" else { return }
        
        NSLog("Opening hook URI: \(url.absoluteString)")
        
        // Call `hk open <uri>` via subprocess
        HKBridge.open(uri: url.absoluteString) { [weak self] result in
            switch result {
            case .success(let message):
                NSLog("Opened: \(message)")
            case .failure(let error):
                self?.showErrorAlert("Failed to open hook URI", error.localizedDescription)
            }
        }
    }
    
    private func showErrorAlert(_ title: String, _ message: String) {
        DispatchQueue.main.async {
            let alert = NSAlert()
            alert.messageText = title
            alert.informativeText = message
            alert.alertStyle = .warning
            alert.runModal()
        }
    }
}
