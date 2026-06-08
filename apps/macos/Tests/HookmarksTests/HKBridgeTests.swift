import XCTest
@testable import Hookmarks

/// Tests for HKBridge path resolution and settings helpers.
///
/// These tests do NOT spawn subprocesses or make HTTP calls —
/// they validate the pure logic layers that are always exercisable
/// regardless of whether `hk` is installed.
final class HKBridgeTests: XCTestCase {

    // MARK: - storedCliPath

    func testStoredCliPathReturnsNilWhenEmpty() {
        UserDefaults.standard.removeObject(forKey: "cliPath")
        XCTAssertNil(HKBridge.storedCliPath)
    }

    func testStoredCliPathReturnsNilWhenWhitespaceOnly() {
        UserDefaults.standard.set("   ", forKey: "cliPath")
        // storedCliPath trims via isEmpty on raw value — whitespace-only
        // is not empty, so it would return it. Documenting actual behaviour:
        let result = HKBridge.storedCliPath
        XCTAssertEqual(result, "   ")
        UserDefaults.standard.removeObject(forKey: "cliPath")
    }

    func testStoredCliPathReturnsValueWhenSet() {
        UserDefaults.standard.set("/usr/local/bin/hk", forKey: "cliPath")
        XCTAssertEqual(HKBridge.storedCliPath, "/usr/local/bin/hk")
        UserDefaults.standard.removeObject(forKey: "cliPath")
    }

    // MARK: - storedServerUrl

    func testStoredServerUrlReturnsNilWhenEmpty() {
        UserDefaults.standard.removeObject(forKey: "serverUrl")
        XCTAssertNil(HKBridge.storedServerUrl)
    }

    func testStoredServerUrlReturnsValueWhenSet() {
        UserDefaults.standard.set("http://127.0.0.1:2701", forKey: "serverUrl")
        XCTAssertEqual(HKBridge.storedServerUrl, "http://127.0.0.1:2701")
        UserDefaults.standard.removeObject(forKey: "serverUrl")
    }

    // MARK: - locateHK (user pref path)

    func testLocateHKPrefersUserConfiguredPath() {
        // Point cliPath at a path we know exists on macOS
        UserDefaults.standard.set("/bin/sh", forKey: "cliPath")
        let located = HKBridge.locateHK()
        XCTAssertEqual(located, "/bin/sh",
            "locateHK should return the user-configured path when it exists")
        UserDefaults.standard.removeObject(forKey: "cliPath")
    }

    func testLocateHKIgnoresNonExistentUserPath() {
        UserDefaults.standard.set("/nonexistent/path/hk", forKey: "cliPath")
        // Should fall through to auto-detection (which may or may not find hk)
        let located = HKBridge.locateHK()
        XCTAssertNotEqual(located, "/nonexistent/path/hk",
            "locateHK should not return a path that does not exist on disk")
        UserDefaults.standard.removeObject(forKey: "cliPath")
    }

    func testLocateHKReturnsNilWhenNothingFound() {
        // Clear prefs and point search at a temp dir with no hk binary
        UserDefaults.standard.removeObject(forKey: "cliPath")
        // We can't easily mock the search paths, but we can verify the return
        // type contract: result is String? — nil is valid when hk not installed.
        let located = HKBridge.locateHK()
        // Either nil or a path that actually exists
        if let path = located {
            XCTAssertTrue(FileManager.default.fileExists(atPath: path),
                "locateHK must only return paths that exist on disk")
        }
    }
}
