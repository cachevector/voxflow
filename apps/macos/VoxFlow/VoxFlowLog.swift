import AppKit
import Foundation
import os

enum VoxFlowLog {
    private static let logger = Logger(subsystem: "com.maskedsyntax.VoxFlow", category: "app")

    private static var logURL: URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("com.maskedsyntax.VoxFlow/voxflow.log")
    }

    static func info(_ message: String) {
        logger.info("\(message, privacy: .public)")
        writeToFile(message)
        NSLog("VoxFlow: \(message)")
    }

    static func error(_ message: String) {
        logger.error("\(message, privacy: .public)")
        writeToFile("ERROR: \(message)")
        NSLog("VoxFlow ERROR: \(message)")
    }

    static func revealLogFile() {
        guard let url = logURL else { return }
        let dir = url.deletingLastPathComponent()
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        if !FileManager.default.fileExists(atPath: url.path) {
            FileManager.default.createFile(atPath: url.path, contents: Data("VoxFlow log\n".utf8))
        }
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }

    private static func writeToFile(_ message: String) {
        guard let url = logURL else { return }
        let dir = url.deletingLastPathComponent()
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let line = "\(ISO8601DateFormatter().string(from: Date())) \(message)\n"
        if FileManager.default.fileExists(atPath: url.path),
           let handle = try? FileHandle(forWritingTo: url) {
            handle.seekToEndOfFile()
            handle.write(Data(line.utf8))
            try? handle.close()
        } else {
            try? line.write(to: url, atomically: true, encoding: .utf8)
        }
    }
}
