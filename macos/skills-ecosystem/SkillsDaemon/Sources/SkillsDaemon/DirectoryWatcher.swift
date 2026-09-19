import Foundation
import CoreServices

public class DirectoryWatcher {
    private let url: URL
    private let handler: () -> Void
    private var stream: FSEventStreamRef?

    public init(url: URL, handler: @escaping () -> Void) {
        self.url = url
        self.handler = handler
    }

    public func start() {
        let callback: FSEventStreamCallback = { (
            streamRef,
            clientCallBackInfo,
            numEvents,
            eventPaths,
            eventFlags,
            eventIds
        ) in
            guard let info = clientCallBackInfo else { return }
            let watcher = Unmanaged<DirectoryWatcher>.fromOpaque(info).takeUnretainedValue()
            NSLog("SkillsDaemon: FSEvents callback triggered recursive change event.")
            watcher.handler()
        }

        var context = FSEventStreamContext(
            version: 0,
            info: UnsafeMutableRawPointer(Unmanaged.passUnretained(self).toOpaque()),
            retain: nil,
            release: nil,
            copyDescription: nil
        )

        let paths = [url.path as NSString] as CFArray
        let latency: TimeInterval = 1.0 // Natively debounces events in a 1-second window

        stream = FSEventStreamCreate(
            nil,
            callback,
            &context,
            paths,
            FSEventStreamEventId(kFSEventStreamEventIdSinceNow),
            latency,
            UInt32(kFSEventStreamCreateFlagUseCFTypes | kFSEventStreamCreateFlagFileEvents)
        )

        if let stream = stream {
            FSEventStreamSetDispatchQueue(stream, DispatchQueue.global(qos: .default))
            FSEventStreamStart(stream)
            NSLog("SkillsDaemon: FSEventStream successfully started watching \(url.path)")
        } else {
            NSLog("SkillsDaemon [Error]: Failed to create FSEventStream for \(url.path)")
        }
    }

    public func stop() {
        if let stream = stream {
            FSEventStreamStop(stream)
            FSEventStreamInvalidate(stream)
            FSEventStreamRelease(stream)
            self.stream = nil
            NSLog("SkillsDaemon: FSEventStream stopped.")
        }
    }

    deinit {
        stop()
    }
}
