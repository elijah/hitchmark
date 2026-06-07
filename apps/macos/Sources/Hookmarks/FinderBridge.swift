//
//  FinderBridge.swift
//  Hookmarks
//
//  Bridge to interact with Finder using AppleScript.
//

import Foundation

struct FinderBridge {
    
    /// Get the path of the currently selected file in Finder
    static func getSelectedFile(completion: @escaping (String?) -> Void) {
        let script = """
        tell application "Finder"
            if exists front Finder window then
                set selectedItems to selection
                if length of selectedItems > 0 then
                    set firstItem to item 1 of selectedItems
                    return POSIX path of (firstItem as alias)
                end if
            end if
        end tell
        """
        
        executeAppleScript(script) { result in
            completion(result)
        }
    }
    
    /// Get the URL of the active Safari tab
    static func getActiveSafariURL(completion: @escaping (String?) -> Void) {
        let script = """
        tell application "Safari"
            if (count of windows) = 0 then
                return ""
            end if
            return URL of current tab of front window
        end tell
        """
        
        executeAppleScript(script) { result in
            completion(result)
        }
    }
    
    /// Open a file at the given path in Finder and select it
    static func revealInFinder(_ path: String) {
        let script = """
        tell application "Finder"
            activate
            set thePath to POSIX file "\(escapeAppleScriptString(path))"
            select thePath
        end tell
        """
        
        executeAppleScript(script) { _ in }
    }
    
    // MARK: - Private
    
    private static func executeAppleScript(
        _ script: String,
        completion: @escaping (String?) -> Void
    ) {
        DispatchQueue.global(qos: .userInitiated).async {
            guard let appleScript = NSAppleScript(source: script) else {
                completion(nil)
                return
            }
            
            var error: NSDictionary?
            let result = appleScript.executeAndReturnError(&error)
            
            if error != nil {
                NSLog("AppleScript error: \(String(describing: error))")
                completion(nil)
                return
            }
            
            let output = result.stringValue ?? ""
            completion(output.isEmpty ? nil : output)
        }
    }
    
    private static func escapeAppleScriptString(_ str: String) -> String {
        return str.replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
    }
}
