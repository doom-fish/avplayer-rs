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
    private let gate: AVPCallbackGate

    init(
        output: AVPlayerItemVideoOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        gate: AVPCallbackGate
    ) {
        self.output = output
        self.callback = callback
        self.gate = gate
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        output?.setDelegate(nil, queue: nil)
        gate.finish()
    }

    func outputMediaDataWillChange(_ sender: AVPlayerItemOutput) {
        send(VideoOutputEventPayload(event: "media_data_will_change"))
    }

    func outputSequenceWasFlushed(_ output: AVPlayerItemOutput) {
        send(VideoOutputEventPayload(event: "sequence_was_flushed"))
    }

    private func send(_ payload: VideoOutputEventPayload) {
        gate.deliverJSON(payload, to: callback)
    }
}

final class MetadataOutputObserverBox: NSObject, AVPlayerItemMetadataOutputPushDelegate {
    private weak var output: AVPlayerItemMetadataOutput?
    private let callback: AVPJsonCallback
    private let gate: AVPCallbackGate

    init(
        output: AVPlayerItemMetadataOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        gate: AVPCallbackGate
    ) {
        self.output = output
        self.callback = callback
        self.gate = gate
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        output?.setDelegate(nil, queue: nil)
        gate.finish()
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
        gate.deliverJSON(payload, to: callback)
    }
}

final class LegibleOutputObserverBox: NSObject, AVPlayerItemLegibleOutputPushDelegate {
    private weak var output: AVPlayerItemLegibleOutput?
    private let callback: AVPJsonObjectsCallback
    private let gate: AVPCallbackGate

    init(
        output: AVPlayerItemLegibleOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonObjectsCallback,
        gate: AVPCallbackGate
    ) {
        self.output = output
        self.callback = callback
        self.gate = gate
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        output?.setDelegate(nil, queue: nil)
        gate.finish()
    }

    func outputSequenceWasFlushed(_ output: AVPlayerItemOutput) {
        gate.deliverJSON(
            LegibleOutputEventPayload(
                event: "sequence_was_flushed",
                itemTime: nil,
                strings: [],
                nativeSampleBufferCount: 0
            ),
            objects: [],
            to: callback
        )
    }

    func legibleOutput(
        _ output: AVPlayerItemLegibleOutput,
        didOutputAttributedStrings strings: [NSAttributedString],
        nativeSampleBuffers nativeSamples: [Any],
        forItemTime itemTime: CMTime
    ) {
        let sampleBuffers: [AnyObject] = nativeSamples.compactMap { sample in
            let object = sample as AnyObject
            guard CFGetTypeID(object) == CMSampleBufferGetTypeID() else { return nil }
            return object
        }
        gate.deliverJSON(
            LegibleOutputEventPayload(
                event: "attributed_strings",
                itemTime: encodeTime(itemTime),
                strings: strings.map(\.string),
                nativeSampleBufferCount: sampleBuffers.count
            ),
            objects: sampleBuffers,
            to: callback
        )
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
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing video-output observer callback")
        return nil
    }
    let output = Unmanaged<AVPPlayerItemVideoOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().videoOutput
    let observer = VideoOutputObserverBox(
        output: output,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_video_output_observer_release")
public func av_player_item_video_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    let observer = Unmanaged<VideoOutputObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
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
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing metadata-output observer callback")
        return nil
    }
    let output = Unmanaged<AVPPlayerItemMetadataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().metadataOutput
    let observer = MetadataOutputObserverBox(
        output: output,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_metadata_output_observer_release")
public func av_player_item_metadata_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    let observer = Unmanaged<MetadataOutputObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}

@_cdecl("av_player_item_legible_output_add_observer")
public func av_player_item_legible_output_add_observer(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPJsonObjectsCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing legible-output observer callback")
        return nil
    }
    let output = Unmanaged<AVPPlayerItemLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().legibleOutput
    let observer = LegibleOutputObserverBox(
        output: output,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_legible_output_observer_release")
public func av_player_item_legible_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    let observer = Unmanaged<LegibleOutputObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}

private func encodeTimedMetadataGroup(_ group: AVTimedMetadataGroup) -> TimedMetadataGroupPayload {
    TimedMetadataGroupPayload(timeRange: encodeTimeRange(group.timeRange), items: group.items.map(avpEncodeMetadataItem))
}
