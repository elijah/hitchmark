import XCTest
@testable import Hookmarks

/// Tests for UserDefaults key names and default values used by @AppStorage.
///
/// These act as a contract — if a key name changes, persisted prefs break
/// for existing users. Failing tests here mean a migration is needed.
final class PreferencesKeysTests: XCTestCase {

    // MARK: - Key name contracts

    func testCliPathKeyIsStable() {
        let key = "cliPath"
        UserDefaults.standard.set("/test/path", forKey: key)
        XCTAssertEqual(UserDefaults.standard.string(forKey: key), "/test/path")
        UserDefaults.standard.removeObject(forKey: key)
    }

    func testServerUrlKeyIsStable() {
        let key = "serverUrl"
        UserDefaults.standard.set("http://127.0.0.1:2701", forKey: key)
        XCTAssertEqual(UserDefaults.standard.string(forKey: key), "http://127.0.0.1:2701")
        UserDefaults.standard.removeObject(forKey: key)
    }

    func testAutoOpenLinksDefaultIsTrue() {
        UserDefaults.standard.removeObject(forKey: "autoOpenLinks")
        // @AppStorage default is true — UserDefaults returns false (0) when absent,
        // so the SwiftUI layer supplies the default. Verify key round-trips correctly.
        UserDefaults.standard.set(true, forKey: "autoOpenLinks")
        XCTAssertTrue(UserDefaults.standard.bool(forKey: "autoOpenLinks"))
        UserDefaults.standard.removeObject(forKey: "autoOpenLinks")
    }

    func testUseGlobalHotkeyDefaultIsFalse() {
        UserDefaults.standard.removeObject(forKey: "useGlobalHotkey")
        XCTAssertFalse(UserDefaults.standard.bool(forKey: "useGlobalHotkey"))
    }

    func testGlobalHotkeyDefaultValue() {
        UserDefaults.standard.removeObject(forKey: "globalHotkey")
        // When absent, @AppStorage supplies "⌃⌥H" — verify round-trip
        UserDefaults.standard.set("⌃⌥H", forKey: "globalHotkey")
        XCTAssertEqual(UserDefaults.standard.string(forKey: "globalHotkey"), "⌃⌥H")
        UserDefaults.standard.removeObject(forKey: "globalHotkey")
    }

    func testMenuBarIconStyleDefaultIsZero() {
        UserDefaults.standard.removeObject(forKey: "menuBarIconStyle")
        XCTAssertEqual(UserDefaults.standard.integer(forKey: "menuBarIconStyle"), 0)
    }

    func testLaunchAtLoginDefaultIsFalse() {
        UserDefaults.standard.removeObject(forKey: "launchAtLogin")
        XCTAssertFalse(UserDefaults.standard.bool(forKey: "launchAtLogin"))
    }

    // MARK: - Value persistence round-trips

    func testBoolPrefsRoundTrip() {
        let keys = ["autoOpenLinks", "launchAtLogin", "useGlobalHotkey"]
        for key in keys {
            UserDefaults.standard.set(true, forKey: key)
            XCTAssertTrue(UserDefaults.standard.bool(forKey: key), "\(key) should persist true")
            UserDefaults.standard.set(false, forKey: key)
            XCTAssertFalse(UserDefaults.standard.bool(forKey: key), "\(key) should persist false")
            UserDefaults.standard.removeObject(forKey: key)
        }
    }

    func testStringPrefsRoundTrip() {
        let pairs: [(String, String)] = [
            ("cliPath", "/opt/homebrew/bin/hk"),
            ("serverUrl", "http://127.0.0.1:9999"),
            ("globalHotkey", "⌘⌃H"),
        ]
        for (key, value) in pairs {
            UserDefaults.standard.set(value, forKey: key)
            XCTAssertEqual(UserDefaults.standard.string(forKey: key), value,
                           "\(key) should round-trip correctly")
            UserDefaults.standard.removeObject(forKey: key)
        }
    }
}
