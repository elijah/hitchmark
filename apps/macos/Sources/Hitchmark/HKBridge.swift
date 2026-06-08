//
//  HKBridge.swift
//  Hookmarks
//
//  Bridge to the `hk` CLI tool — subprocess for local ops,
//  HTTP for link queries when `hk serve` is running.
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
    
    // MARK: - Public API
    
    /// Run `hk file <path>` to convert a file path to a hook:// URI
    static func fileToURI(_ path: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        // Try HTTP server first if configured
        if let serverUrl = storedServerUrl {
            let encodedPath = path.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? path
            httpGet("\(serverUrl)/uri?path=\(encodedPath)") { result in
                switch result {
                case .success(let data):
                    if let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                       let uri = json["uri"] as? String {
                        completion(.success(uri))
                        return
                    }
                    fallthrough
                case .failure:
                    runCommand("file", [path], completion: completion)
                }
            }
        } else {
            runCommand("file", [path], completion: completion)
        }
    }
    
    /// Run `hk open <uri>` to resolve and open a hook:// URI (always subprocess — needs OS)
    static func open(uri: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        runCommand("open", [uri], completion: completion)
    }
    
    /// Run `hk list <uri> --json` to query links for a resource
    static func list(uri: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        if let serverUrl = storedServerUrl {
            let encodedUri = uri.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? uri
            httpGet("\(serverUrl)/links?uri=\(encodedUri)") { result in
                switch result {
                case .success(let data):
                    if let str = String(data: data, encoding: .utf8) {
                        completion(.success(str))
                        return
                    }
                    fallthrough
                case .failure:
                    runCommand("list", [uri, "--json"], completion: completion)
                }
            }
        } else {
            runCommand("list", [uri, "--json"], completion: completion)
        }
    }
    
    /// Run `hk link <uri-a> <uri-b> [--note "..."]`
    static func link(
        uriA: String,
        uriB: String,
        note: String? = nil,
        completion: @escaping (Result<String, HKBridgeError>) -> Void
    ) {
        if let serverUrl = storedServerUrl {
            var body: [String: Any] = ["uri_a": uriA, "uri_b": uriB]
            if let note = note { body["note"] = note }
            httpPost("\(serverUrl)/links", body: body) { result in
                switch result {
                case .success:
                    completion(.success(""))
                case .failure:
                    var args = [uriA, uriB]
                    if let note = note { args += ["--note", note] }
                    runCommand("link", args, completion: completion)
                }
            }
        } else {
            var args = [uriA, uriB]
            if let note = note { args += ["--note", note] }
            runCommand("link", args, completion: completion)
        }
    }
    
    /// Run `hk purple <file> --format json`
    static func purple(filePath: String, completion: @escaping (Result<String, HKBridgeError>) -> Void) {
        runCommand("purple", [filePath, "--format", "json"], completion: completion)
    }
    
    // MARK: - Settings helpers
    
    /// The stored `cliPath` from UserDefaults, or nil if empty
    static var storedCliPath: String? {
        let v = UserDefaults.standard.string(forKey: "cliPath") ?? ""
        return v.isEmpty ? nil : v
    }
    
    /// The stored `serverUrl` from UserDefaults, or nil if empty
    static var storedServerUrl: String? {
        let v = UserDefaults.standard.string(forKey: "serverUrl") ?? ""
        return v.isEmpty ? nil : v
    }
    
    // MARK: - Subprocess
    
    private static func runCommand(
        _ subcommand: String,
        _ args: [String],
        completion: @escaping (Result<String, HKBridgeError>) -> Void
    ) {
        DispatchQueue.global(qos: .userInitiated).async {
            guard let hkPath = locateHK() else {
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
    
    /// Find `hk` — checks user pref first, then common install locations.
    static func locateHK() -> String? {
        // 1. User-configured path takes priority
        if let custom = storedCliPath,
           FileManager.default.fileExists(atPath: custom) {
            return custom
        }
        
        // 2. Common install locations
        let searchPaths = [
            "/usr/local/bin/hk",
            "\(NSHomeDirectory())/.cargo/bin/hk",
            "/opt/homebrew/bin/hk",
            "/usr/bin/hk",
            "/usr/local/opt/hookmarks/bin/hk"
        ]
        return searchPaths.first { FileManager.default.fileExists(atPath: $0) }
    }
    
    // MARK: - HTTP helpers
    
    private static func httpGet(
        _ urlString: String,
        completion: @escaping (Result<Data, HKBridgeError>) -> Void
    ) {
        guard let url = URL(string: urlString) else {
            completion(.failure(.invalidOutput))
            return
        }
        var request = URLRequest(url: url, timeoutInterval: 3)
        request.httpMethod = "GET"
        URLSession.shared.dataTask(with: request) { data, response, _ in
            if let data = data,
               (response as? HTTPURLResponse)?.statusCode == 200 {
                completion(.success(data))
            } else {
                completion(.failure(.failed("HTTP request failed")))
            }
        }.resume()
    }
    
    private static func httpPost(
        _ urlString: String,
        body: [String: Any],
        completion: @escaping (Result<Data, HKBridgeError>) -> Void
    ) {
        guard let url = URL(string: urlString),
              let bodyData = try? JSONSerialization.data(withJSONObject: body) else {
            completion(.failure(.invalidOutput))
            return
        }
        var request = URLRequest(url: url, timeoutInterval: 3)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = bodyData
        URLSession.shared.dataTask(with: request) { data, response, _ in
            let status = (response as? HTTPURLResponse)?.statusCode ?? 0
            if let data = data, status == 200 || status == 201 {
                completion(.success(data))
            } else {
                completion(.failure(.failed("HTTP \(status)")))
            }
        }.resume()
    }
}
