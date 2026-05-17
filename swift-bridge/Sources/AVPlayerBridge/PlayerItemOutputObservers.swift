import AVFoundation
import Foundation

private struct VideoOutputEventPayload: Codable {
    let event: String
}

private struct TimedMetadataGroupPayload: Codable {
    let timeRange: TimeRangePayload
    let items: [MetadataItemPayload]
}

private struct MetadataOutputEventPayload: Codable {
    let event: String
    let groups: [TimedMetadataGroupPayload]
    let trackPresent: Bool
}

private struct LegibleOutputEventPayload: Codable {
    let event: String
    let itemTime: TimePayload?
    let strings: [String]
    let nativeSampleBufferCount: Int
}

final class VideoOutputObserverBox: NSObject, AVPlayerItemOutputPullDelegate {
    private weak var output: AVPlayerItemVideoOutput?
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        output: AVPlayerItemVideoOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.output = output
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        output?.setDelegate(nil, queue: nil)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    func outputMediaDataWillChange(_ sender: AVPlayerItemOutput) {
        send(VideoOutputEventPayload(event: "media_data_will_change"))
    }

    func outputSequenceWasFlushed(_ output: AVPlayerItemOutput) {
        send(VideoOutputEventPayload(event: "sequence_was_flushed"))
    }

    private func send(_ payload: VideoOutputEventPayload) {
        guard !disposed else { return }
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
    }
}

final class MetadataOutputObserverBox: NSObject, AVPlayerItemMetadataOutputPushDelegate {
    private weak var output: AVPlayerItemMetadataOutput?
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        output: AVPlayerItemMetadataOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.output = output
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        output?.setDelegate(nil, queue: nil)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    func outputSequenceWasFlushed(_ output: AVPlayerItemOutput) {
        send(MetadataOutputEventPayload(event: "sequence_was_flushed", groups: [], trackPresent: false))
    }

    func metadataOutput(
        _ output: AVPlayerItemMetadataOutput,
        didOutputTimedMetadataGroups groups: [AVTimedMetadataGroup],
        from track: AVPlayerItemTrack?
    ) {
        send(
            MetadataOutputEventPayload(
                event: "timed_metadata_groups",
                groups: groups.map(encodeTimedMetadataGroup),
                trackPresent: track != nil
            )
        )
    }

    private func send(_ payload: MetadataOutputEventPayload) {
        guard !disposed else { return }
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
    }
}

final class LegibleOutputObserverBox: NSObject, AVPlayerItemLegibleOutputPushDelegate {
    private weak var output: AVPlayerItemLegibleOutput?
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        output: AVPlayerItemLegibleOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.output = output
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        output?.setDelegate(nil, queue: nil)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    func outputSequenceWasFlushed(_ output: AVPlayerItemOutput) {
        send(
            LegibleOutputEventPayload(
                event: "sequence_was_flushed",
                itemTime: nil,
                strings: [],
                nativeSampleBufferCount: 0
            )
        )
    }

    func legibleOutput(
        _ output: AVPlayerItemLegibleOutput,
        didOutputAttributedStrings strings: [NSAttributedString],
        nativeSampleBuffers nativeSamples: [Any],
        forItemTime itemTime: CMTime
    ) {
        send(
            LegibleOutputEventPayload(
                event: "attributed_strings",
                itemTime: encodeTime(itemTime),
                strings: strings.map(\.string),
                nativeSampleBufferCount: nativeSamples.count
            )
        )
    }

    private func send(_ payload: LegibleOutputEventPayload) {
        guard !disposed else { return }
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
    }
}

@_cdecl("av_player_item_video_output_add_observer")
public func av_player_item_video_output_add_observer(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing video-output observer callback")
        return nil
    }
    let output = Unmanaged<AVPPlayerItemVideoOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().videoOutput
    let observer = VideoOutputObserverBox(
        output: output,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_video_output_observer_release")
public func av_player_item_video_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<VideoOutputObserverBox>.fromOpaque(observerPtr).release()
}

@_cdecl("av_player_item_video_output_request_notification_of_media_data_change")
public func av_player_item_video_output_request_notification_of_media_data_change(
    _ outputPtr: UnsafeMutableRawPointer,
    _ interval: Double
) {
    let output = Unmanaged<AVPPlayerItemVideoOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().videoOutput
    output.requestNotificationOfMediaDataChange(withAdvanceInterval: interval)
}

@_cdecl("av_player_item_metadata_output_add_observer")
public func av_player_item_metadata_output_add_observer(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing metadata-output observer callback")
        return nil
    }
    let output = Unmanaged<AVPPlayerItemMetadataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().metadataOutput
    let observer = MetadataOutputObserverBox(
        output: output,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_metadata_output_observer_release")
public func av_player_item_metadata_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<MetadataOutputObserverBox>.fromOpaque(observerPtr).release()
}

@_cdecl("av_player_item_legible_output_add_observer")
public func av_player_item_legible_output_add_observer(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing legible-output observer callback")
        return nil
    }
    let output = Unmanaged<AVPPlayerItemLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().legibleOutput
    let observer = LegibleOutputObserverBox(
        output: output,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_legible_output_observer_release")
public func av_player_item_legible_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<LegibleOutputObserverBox>.fromOpaque(observerPtr).release()
}

private func encodeTimedMetadataGroup(_ group: AVTimedMetadataGroup) -> TimedMetadataGroupPayload {
    TimedMetadataGroupPayload(timeRange: encodeTimeRange(group.timeRange), items: group.items.map(avpEncodeMetadataItem))
}
