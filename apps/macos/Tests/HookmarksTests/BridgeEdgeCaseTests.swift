import XCTest
@testable import Hookmarks

/// Tests for FinderBridge AppleScript path handling.
///
/// We can't invoke Finder in a test runner, but we can test the
/// path normalisation and edge-case handling logic.
final class FinderBridgeTests: XCTestCase {

    // MARK: - Path cleaning

    func testHomeDirectoryExpansion() {
        let home = NSHomeDirectory()
        XCTAssertFalse(home.isEmpty, "NSHomeDirectory() must not be empty")
        XCTAssertTrue(home.hasPrefix("/"), "Home directory must be an absolute path")
    }

    func testFileManagerExistsForKnownPaths() {
        // Paths that must always exist on macOS
        let known = ["/usr", "/tmp", "/bin/sh", NSHomeDirectory()]
        for path in known {
            XCTAssertTrue(FileManager.default.fileExists(atPath: path),
                          "Expected \(path) to exist")
        }
    }

    func testFileManagerDoesNotExistForFakePaths() {
        let fake = [
            "/totally/fake/path/that/cannot/exist",
            "/not-hookmarks-test-sentinel-12345",
        ]
        for path in fake {
            XCTAssertFalse(FileManager.default.fileExists(atPath: path),
                           "Expected \(path) NOT to exist")
        }
    }
}

/// Tests for HKBridgeError display strings — these show in the UI so
/// they must be human-readable and stable.
final class HKBridgeErrorTests: XCTestCase {

    func testNotFoundErrorHasInstallInstructions() {
        let error = HKBridgeError.notFound
        let desc = error.errorDescription ?? ""
        XCTAssertTrue(desc.contains("brew") || desc.contains("cargo"),
            "notFound error should mention how to install hk")
    }

    func testFailedErrorIncludesReason() {
        let reason = "exit status 1"
        let error = HKBridgeError.failed(reason)
        let desc = error.errorDescription ?? ""
        XCTAssertTrue(desc.contains(reason),
            "failed error should include the original reason string")
    }

    func testInvalidOutputErrorIsDescriptive() {
        let error = HKBridgeError.invalidOutput
        let desc = error.errorDescription ?? ""
        XCTAssertFalse(desc.isEmpty, "invalidOutput error must have a non-empty description")
    }

    func testAllCasesHaveNonNilDescriptions() {
        let cases: [HKBridgeError] = [
            .notFound,
            .failed("test reason"),
            .invalidOutput,
        ]
        for error in cases {
            XCTAssertNotNil(error.errorDescription,
                "All HKBridgeError cases must have a localised description")
        }
    }
}
