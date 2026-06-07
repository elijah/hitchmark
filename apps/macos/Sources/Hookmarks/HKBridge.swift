//
//  HKBridge.swift
//  Hookmarks
//
//  Bridge to call the `hk` CLI tool via subprocess.
//  This allows the macOS app to leverage the core Rust library.
//

import Foundation

enum HKBridgeError: LocalizedError {
    case notFound
    case failed(String)
    case invalidOutput
    
    var errorDescription: String? {
        switch self {
        case .notFound:
            return "hk command not found. Install Hookmarks CLI with: brew install hookmarks"
        case .failed(let reason):
            return "hk failed: \(reason)"
        case .invalidOutput:
            return "hk returned invalid output"
        }
    }
}

struct HKBridge {
    
    /// Run `hk file <path>` to convert a file path to a hook:// URI
    static func fileToURI(_ path: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        runCommand("file", [path]) { result in
            switch result {
            case .success(let output):
                let uri = output.trimmingCharacters(in: .whitespacesAndNewlines)
                completion(.success(uri))
            case .failure(let error):
                completion(.failure(error))
            }
        }
    }
    
    /// Run `hk open <uri>` to resolve and open a hook:// URI
    static func open(uri: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        runCommand("open", [uri]) { result in
            completion(result)
        }
    }
    
    /// Run `hk list <uri>` to query links for a resource
    static func list(uri: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        runCommand("list", [uri]) { result in
            completion(result)
        }
    }
    
    /// Run `hk link <uri-a> <uri-b> [--note "..."]`
    static func link(
        uriA: String,
        uriB: String,
        note: String? = nil,
        completion: @escaping (Result<String, HKBridgeError>) -> Void
    ) {
        var args = [uriA, uriB]
        if let note = note {
            args.append("--note")
            args.append(note)
        }
        runCommand("link", args) { result in
            completion(result)
        }
    }
    
    /// Run `hk purple <file> --format json` to annotate with purple numbers
    static func purple(filePath: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        runCommand("purple", [filePath, "--format", "json"]) { result in
            completion(result)
        }
    }
    
    // MARK: - Private
    
    private static func runCommand(
        _ subcommand: String,
        _ args: [String],
        completion: @escaping (Result<String, HKBridgeError>) -> Void
    ) {
        DispatchQueue.global(qos: .userInitiated).async {
            let hkPath = locateHK()
            
            guard let hkPath = hkPath else {
                completion(.failure(.notFound))
                return
            }
            
            let process = Process()
            process.executableURL = URL(fileURLWithPath: hkPath)
            process.arguments = [subcommand] + args
            
            let pipe = Pipe()
            let errorPipe = Pipe()
            process.standardOutput = pipe
            process.standardError = errorPipe
            
            do {
                try process.run()
                process.waitUntilExit()
                
                let data = pipe.fileHandleForReading.readDataToEndOfFile()
                let output = String(data: data, encoding: .utf8) ?? ""
                
                if process.terminationStatus == 0 {
                    completion(.success(output))
                } else {
                    let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
                    let errorOutput = String(data: errorData, encoding: .utf8) ?? "Unknown error"
                    completion(.failure(.failed(errorOutput)))
                }
            } catch {
                completion(.failure(.failed(error.localizedDescription)))
            }
        }
    }
    
    /// Find `hk` in common locations: /usr/local/bin, ~/.cargo/bin, brew
    private static func locateHK() -> String? {
        let searchPaths = [
            "/usr/local/bin/hk",
            "\(NSHomeDirectory())/.cargo/bin/hk",
            "/opt/homebrew/bin/hk",
            "/usr/local/opt/hookmarks/bin/hk"
        ]
        
        for path in searchPaths {
            if FileManager.default.fileExists(atPath: path) {
                return path
            }
        }
        
        return nil
    }
}
