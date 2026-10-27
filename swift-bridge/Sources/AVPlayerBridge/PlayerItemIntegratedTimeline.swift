import AVPlayerABI
import AVFoundation
import Foundation

struct PlayerItemIntegratedTimelineInfoPayload: AVPBridge {
    let currentTime: TimePayload
    let currentDate: String?
}

struct PlayerItemIntegratedTimelineSegmentPayload: AVPBridge {
    let segmentTypeRaw: Int
    let timeMappingSource: TimeRangePayload
    let timeMappingTarget: TimeRangePayload
    let loadedTimeRanges: [TimeRangePayload]
    let startDate: String?
    let interstitialEvent: PlayerInterstitialEventInfoPayload?
}

struct PlayerItemIntegratedTimelineSnapshotPayload: AVPBridge {
    let duration: TimePayload
    let currentTime: TimePayload
    let currentDate: String?
    let currentSegmentIndex: Int?
    let segments: [PlayerItemIntegratedTimelineSegmentPayload]
}

struct PlayerItemIntegratedTimelineSegmentOffsetPayload: AVPBridge {
    let segment: PlayerItemIntegratedTimelineSegmentPayload
    let offset: TimePayload
}

struct PlayerIntegratedTimelineOutOfSyncPayload: AVPBridge {
    let reason: String
}

@available(macOS 15.0, *)
class AVPPlayerItemIntegratedTimelineBox: NSObject {
    let timeline: AVPlayerItemIntegratedTimeline

    init(timeline: AVPlayerItemIntegratedTimeline) {
        self.timeline = timeline
    }
}

@available(macOS 15.0, *)
class AVPPlayerItemIntegratedTimelineSnapshotBox: NSObject {
    let snapshot: AVPlayerItemIntegratedTimelineSnapshot

    init(snapshot: AVPlayerItemIntegratedTimelineSnapshot) {
        self.snapshot = snapshot
    }
}

@available(macOS 15.0, *)
class AVPPlayerItemIntegratedTimelineSegmentBox: NSObject {
    let segment: AVPlayerItemSegment

    init(segment: AVPlayerItemSegment) {
        self.segment = segment
    }
}

@available(macOS 15.0, *)
class AVPPlayerItemIntegratedTimelineObserverBox: NSObject {
    fileprivate let gate: AVPCallbackGate

    init(gate: AVPCallbackGate) {
        self.gate = gate
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        teardown()
        gate.finish()
    }

    fileprivate func teardown() {}
}

@available(macOS 15.0, *)
final class AVPPlayerItemIntegratedTimelineTimeObserverBox: AVPPlayerItemIntegratedTimelineObserverBox {
    private var task: Task<Void, Never>?
    private let callback: AVPPeriodicTimeCallback

    init(callback: @escaping AVPPeriodicTimeCallback, gate: AVPCallbackGate) {
        self.callback = callback
        super.init(gate: gate)
    }

    override fileprivate func teardown() {
        task?.cancel()
        task = nil
    }

    func startPeriodic(timeline: AVPlayerItemIntegratedTimeline, interval: CMTime) {
        task = Task { [gate, callback] in
            for await time in timeline.periodicTimes(forInterval: interval) {
                guard avpDeliverIntegratedTimelineTime(time, gate: gate, callback: callback) else { break }
            }
        }
    }

    func startBoundary(
        timeline: AVPlayerItemIntegratedTimeline,
        segment: AVPlayerItemSegment,
        offsetsIntoSegment: [CMTime]
    ) {
        task = Task { [gate, callback] in
            for await time in timeline.boundaryTimes(for: segment, offsetsIntoSegment: offsetsIntoSegment) {
                guard avpDeliverIntegratedTimelineTime(time, gate: gate, callback: callback) else { break }
            }
        }
    }
}

private func avpDeliverIntegratedTimelineTime(
    _ time: CMTime,
    gate: AVPCallbackGate,
    callback: AVPPeriodicTimeCallback
) -> Bool {
    gate.run(false) { context in
        let encoded = encodeTime(time)
        callback(
            context,
            encoded.value ?? 0,
            encoded.timescale ?? 0,
            encoded.kind
        )
        return true
    }
}

@available(macOS 15.0, *)
final class AVPPlayerItemIntegratedTimelineNotificationObserverBox: AVPPlayerItemIntegratedTimelineObserverBox {
    private var token: NSObjectProtocol?

    init(
        timeline: AVPlayerItemIntegratedTimeline,
        callback: @escaping AVPValueCallback,
        gate: AVPCallbackGate
    ) {
        super.init(gate: gate)
        token = NotificationCenter.default.addObserver(
            forName: AVPlayerItemIntegratedTimeline.snapshotsOutOfSyncNotification,
            object: timeline,
            queue: nil
        ) { [gate] notification in
            let reason = notification.userInfo?[AVPlayerItemIntegratedTimeline.snapshotsOutOfSyncReasonKey] as? String
            gate.deliverValue(PlayerIntegratedTimelineOutOfSyncPayload(reason: reason ?? ""), to: callback)
        }
    }

    override fileprivate func teardown() {
        if let token {
            NotificationCenter.default.removeObserver(token)
            self.token = nil
        }
    }
}

private func avpIntegratedTimelineUnavailable(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) {
    outErrorMessage?.pointee = ffiString(
        "AVPlayerItemIntegratedTimeline requires macOS 15.0 or newer"
    )
}

private func avpParseIntegratedTimelineDate(_ string: String) -> Date? {
    ISO8601DateFormatter().date(from: string)
}

@_cdecl("av_player_item_copy_integrated_timeline")
public func av_player_item_copy_integrated_timeline(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    return Unmanaged.passRetained(AVPPlayerItemIntegratedTimelineBox(timeline: item.integratedTimeline)).toOpaque()
}

@_cdecl("av_player_item_integrated_timeline_release")
public func av_player_item_integrated_timeline_release(_ timelinePtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 15.0, *) else { return }
    guard let timelinePtr else { return }
    Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).release()
}

@_cdecl("av_player_item_integrated_timeline_info_value")
public func av_player_item_integrated_timeline_info_value(
    _ timelinePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    let timeline = Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).takeUnretainedValue().timeline
    do {
        return try avpEncodeValue(
            PlayerItemIntegratedTimelineInfoPayload(
                currentTime: encodeTime(timeline.currentTime),
                currentDate: avpISO8601String(timeline.currentDate)
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_integrated_timeline_copy_current_snapshot")
public func av_player_item_integrated_timeline_copy_current_snapshot(
    _ timelinePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    let timeline = Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).takeUnretainedValue().timeline
    return Unmanaged.passRetained(AVPPlayerItemIntegratedTimelineSnapshotBox(snapshot: timeline.currentSnapshot)).toOpaque()
}

@_cdecl("av_player_item_integrated_timeline_seek_to_time")
public func av_player_item_integrated_timeline_seek_to_time(
    _ timelinePtr: UnsafeMutableRawPointer,
    _ timeValue: Int64,
    _ timeTimescale: Int32,
    _ timeKind: Int32,
    _ beforeValue: Int64,
    _ beforeTimescale: Int32,
    _ beforeKind: Int32,
    _ afterValue: Int64,
    _ afterTimescale: Int32,
    _ afterKind: Int32,
    _ outSuccess: UnsafeMutablePointer<Bool>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return AVP_OPERATION_FAILED
    }
    let timeline = Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).takeUnretainedValue().timeline
    let time = cmTime(value: timeValue, timescale: timeTimescale, kind: timeKind)
    let before = cmTime(value: beforeValue, timescale: beforeTimescale, kind: beforeKind)
    let after = cmTime(value: afterValue, timescale: afterTimescale, kind: afterKind)
    if let message = avpSeekTimeError(time, what: "seek time")
        ?? avpToleranceError(before, what: "tolerance before")
        ?? avpToleranceError(after, what: "tolerance after") {
        outErrorMessage?.pointee = ffiString(message)
        return AVP_INVALID_ARGUMENT
    }
    let semaphore = DispatchSemaphore(value: 0)
    var success = false
    timeline.seek(to: time, toleranceBefore: before, toleranceAfter: after) { didSeek in
        success = didSeek
        semaphore.signal()
    }
    guard semaphore.wait(timeout: .now() + .seconds(10)) == .success else {
        outErrorMessage?.pointee = ffiString("timed out waiting for integrated timeline seek(to:) completion")
        return AVP_OPERATION_FAILED
    }
    outSuccess?.pointee = success
    return AVP_OK
}

@_cdecl("av_player_item_integrated_timeline_seek_to_date")
public func av_player_item_integrated_timeline_seek_to_date(
    _ timelinePtr: UnsafeMutableRawPointer,
    _ datePtr: UnsafePointer<AVPValue>,
    _ outSuccess: UnsafeMutablePointer<Bool>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return AVP_OPERATION_FAILED
    }
    let timeline = Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).takeUnretainedValue().timeline
    guard let date = avpParseIntegratedTimelineDate(String(bridge: datePtr)) else {
        outErrorMessage?.pointee = ffiString("invalid ISO-8601 date string")
        return AVP_INVALID_ARGUMENT
    }
    let semaphore = DispatchSemaphore(value: 0)
    var success = false
    timeline.seek(to: date) { didSeek in
        success = didSeek
        semaphore.signal()
    }
    guard semaphore.wait(timeout: .now() + .seconds(10)) == .success else {
        outErrorMessage?.pointee = ffiString("timed out waiting for integrated timeline seek(to:) completion")
        return AVP_OPERATION_FAILED
    }
    outSuccess?.pointee = success
    return AVP_OK
}

@_cdecl("av_player_item_integrated_timeline_add_periodic_time_observer")
public func av_player_item_integrated_timeline_add_periodic_time_observer(
    _ timelinePtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ callback: AVPPeriodicTimeCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing integrated timeline periodic observer callback")
        return nil
    }
    let interval = cmTime(value: value, timescale: timescale, kind: kind)
    guard interval.isNumeric, interval > .zero else {
        outErrorMessage?.pointee = ffiString("time-observer interval must be a positive numeric time")
        return nil
    }
    let timeline = Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).takeUnretainedValue().timeline
    let observer = AVPPlayerItemIntegratedTimelineTimeObserverBox(callback: callback, gate: gate)
    observer.startPeriodic(timeline: timeline, interval: interval)
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_integrated_timeline_add_boundary_time_observer")
public func av_player_item_integrated_timeline_add_boundary_time_observer(
    _ timelinePtr: UnsafeMutableRawPointer,
    _ segmentPtr: UnsafeMutableRawPointer,
    _ offsetsValue: UnsafePointer<AVPValue>,
    _ callback: AVPPeriodicTimeCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing integrated timeline boundary observer callback")
        return nil
    }
    let timeline = Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).takeUnretainedValue().timeline
    let segment = Unmanaged<AVPPlayerItemIntegratedTimelineSegmentBox>.fromOpaque(segmentPtr).takeUnretainedValue().segment
    let offsetsPayload: [TimePayload]
    do {
        offsetsPayload = try avpDecodeValue(offsetsValue, as: [TimePayload].self)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
    let observer = AVPPlayerItemIntegratedTimelineTimeObserverBox(callback: callback, gate: gate)
    observer.startBoundary(
        timeline: timeline,
        segment: segment,
        offsetsIntoSegment: offsetsPayload.map(cmTime(from:))
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_integrated_timeline_add_out_of_sync_observer")
public func av_player_item_integrated_timeline_add_out_of_sync_observer(
    _ timelinePtr: UnsafeMutableRawPointer,
    _ callback: AVPValueCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing integrated timeline out-of-sync callback")
        return nil
    }
    let timeline = Unmanaged<AVPPlayerItemIntegratedTimelineBox>.fromOpaque(timelinePtr).takeUnretainedValue().timeline
    let observer = AVPPlayerItemIntegratedTimelineNotificationObserverBox(
        timeline: timeline,
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_integrated_timeline_observer_release")
public func av_player_item_integrated_timeline_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 15.0, *) else { return }
    guard let observerPtr else { return }
    let observer = Unmanaged<AVPPlayerItemIntegratedTimelineObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}

@_cdecl("av_player_item_integrated_timeline_snapshot_release")
public func av_player_item_integrated_timeline_snapshot_release(_ snapshotPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 15.0, *) else { return }
    guard let snapshotPtr else { return }
    Unmanaged<AVPPlayerItemIntegratedTimelineSnapshotBox>.fromOpaque(snapshotPtr).release()
}

@_cdecl("av_player_item_integrated_timeline_snapshot_info_value")
public func av_player_item_integrated_timeline_snapshot_info_value(
    _ snapshotPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    let snapshot = Unmanaged<AVPPlayerItemIntegratedTimelineSnapshotBox>.fromOpaque(snapshotPtr).takeUnretainedValue().snapshot
    do {
        return try avpEncodeValue(encodeIntegratedTimelineSnapshot(snapshot))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_integrated_timeline_snapshot_copy_current_segment")
public func av_player_item_integrated_timeline_snapshot_copy_current_segment(
    _ snapshotPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else { return nil }
    let snapshot = Unmanaged<AVPPlayerItemIntegratedTimelineSnapshotBox>.fromOpaque(snapshotPtr).takeUnretainedValue().snapshot
    guard let segment = snapshot.currentSegment else { return nil }
    return Unmanaged.passRetained(AVPPlayerItemIntegratedTimelineSegmentBox(segment: segment)).toOpaque()
}

@_cdecl("av_player_item_integrated_timeline_snapshot_segment_count")
public func av_player_item_integrated_timeline_snapshot_segment_count(
    _ snapshotPtr: UnsafeMutableRawPointer
) -> Int {
    guard #available(macOS 15.0, *) else { return 0 }
    let snapshot = Unmanaged<AVPPlayerItemIntegratedTimelineSnapshotBox>.fromOpaque(snapshotPtr).takeUnretainedValue().snapshot
    return snapshot.segments.count
}

@_cdecl("av_player_item_integrated_timeline_snapshot_copy_segment_at_index")
public func av_player_item_integrated_timeline_snapshot_copy_segment_at_index(
    _ snapshotPtr: UnsafeMutableRawPointer,
    _ index: Int
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else { return nil }
    let snapshot = Unmanaged<AVPPlayerItemIntegratedTimelineSnapshotBox>.fromOpaque(snapshotPtr).takeUnretainedValue().snapshot
    guard index >= 0, index < snapshot.segments.count else { return nil }
    return Unmanaged.passRetained(AVPPlayerItemIntegratedTimelineSegmentBox(segment: snapshot.segments[index])).toOpaque()
}

@_cdecl("av_player_item_integrated_timeline_snapshot_segment_and_offset_value")
public func av_player_item_integrated_timeline_snapshot_segment_and_offset_value(
    _ snapshotPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    let snapshot = Unmanaged<AVPPlayerItemIntegratedTimelineSnapshotBox>.fromOpaque(snapshotPtr).takeUnretainedValue().snapshot
    do {
        let (segment, offset) = snapshot.segmentAndOffsetIntoSegment(forTimelineTime: cmTime(value: value, timescale: timescale, kind: kind))
        return try avpEncodeValue(
            PlayerItemIntegratedTimelineSegmentOffsetPayload(
                segment: encodeIntegratedTimelineSegment(segment),
                offset: encodeTime(offset)
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_integrated_timeline_segment_release")
public func av_player_item_integrated_timeline_segment_release(_ segmentPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 15.0, *) else { return }
    guard let segmentPtr else { return }
    Unmanaged<AVPPlayerItemIntegratedTimelineSegmentBox>.fromOpaque(segmentPtr).release()
}

@_cdecl("av_player_item_integrated_timeline_segment_info_value")
public func av_player_item_integrated_timeline_segment_info_value(
    _ segmentPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    let segment = Unmanaged<AVPPlayerItemIntegratedTimelineSegmentBox>.fromOpaque(segmentPtr).takeUnretainedValue().segment
    do {
        return try avpEncodeValue(encodeIntegratedTimelineSegment(segment))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_integrated_timeline_snapshots_out_of_sync_notification")
public func av_player_integrated_timeline_snapshots_out_of_sync_notification(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    do {
        return try avpEncodeValue(AVPlayerItemIntegratedTimeline.snapshotsOutOfSyncNotification.rawValue)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_integrated_timeline_snapshots_out_of_sync_reason_key")
public func av_player_integrated_timeline_snapshots_out_of_sync_reason_key(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    do {
        return try avpEncodeValue(AVPlayerItemIntegratedTimeline.snapshotsOutOfSyncReasonKey)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed")
public func av_player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    do {
        return try avpEncodeValue(AVPlayerIntegratedTimelineSnapshotsOutOfSyncReason.segmentsChanged.rawValue)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed")
public func av_player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    do {
        return try avpEncodeValue(AVPlayerIntegratedTimelineSnapshotsOutOfSyncReason.currentSegmentChanged.rawValue)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed")
public func av_player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        avpIntegratedTimelineUnavailable(outErrorMessage)
        return nil
    }
    do {
        return try avpEncodeValue(AVPlayerIntegratedTimelineSnapshotsOutOfSyncReason.loadedTimeRangesChanged.rawValue)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@available(macOS 15.0, *)
private func encodeIntegratedTimelineSnapshot(
    _ snapshot: AVPlayerItemIntegratedTimelineSnapshot
) -> PlayerItemIntegratedTimelineSnapshotPayload {
    let segments = snapshot.segments.map(encodeIntegratedTimelineSegment)
    let currentSegmentIndex = snapshot.currentSegment.flatMap { currentSegment in
        snapshot.segments.firstIndex { $0 === currentSegment }
    }
    return PlayerItemIntegratedTimelineSnapshotPayload(
        duration: encodeTime(snapshot.duration),
        currentTime: encodeTime(snapshot.currentTime),
        currentDate: avpISO8601String(snapshot.currentDate),
        currentSegmentIndex: currentSegmentIndex,
        segments: segments
    )
}

@available(macOS 15.0, *)
private func encodeIntegratedTimelineSegment(
    _ segment: AVPlayerItemSegment
) -> PlayerItemIntegratedTimelineSegmentPayload {
    PlayerItemIntegratedTimelineSegmentPayload(
        segmentTypeRaw: segment.segmentType.rawValue,
        timeMappingSource: encodeTimeRange(segment.timeMapping.source),
        timeMappingTarget: encodeTimeRange(segment.timeMapping.target),
        loadedTimeRanges: segment.loadedTimeRanges.map(encodeTimeRange),
        startDate: avpISO8601String(segment.startDate),
        interstitialEvent: segment.interstitialEvent.map { encodeInterstitialEventSummary($0) }
    )
}

extension PlayerItemIntegratedTimelineInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            currentTime.binaryValue,
            currentDate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            currentTime: try binaryValue.field(0),
            currentDate: try binaryValue.field(1)
        )
    }
}

extension PlayerItemIntegratedTimelineSegmentPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            segmentTypeRaw.binaryValue,
            timeMappingSource.binaryValue,
            timeMappingTarget.binaryValue,
            loadedTimeRanges.binaryValue,
            startDate.binaryValue,
            interstitialEvent.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            segmentTypeRaw: try binaryValue.field(0),
            timeMappingSource: try binaryValue.field(1),
            timeMappingTarget: try binaryValue.field(2),
            loadedTimeRanges: try binaryValue.field(3),
            startDate: try binaryValue.field(4),
            interstitialEvent: try binaryValue.field(5)
        )
    }
}

extension PlayerItemIntegratedTimelineSnapshotPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            duration.binaryValue,
            currentTime.binaryValue,
            currentDate.binaryValue,
            currentSegmentIndex.binaryValue,
            segments.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            duration: try binaryValue.field(0),
            currentTime: try binaryValue.field(1),
            currentDate: try binaryValue.field(2),
            currentSegmentIndex: try binaryValue.field(3),
            segments: try binaryValue.field(4)
        )
    }
}

extension PlayerItemIntegratedTimelineSegmentOffsetPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            segment.binaryValue,
            offset.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            segment: try binaryValue.field(0),
            offset: try binaryValue.field(1)
        )
    }
}

extension PlayerIntegratedTimelineOutOfSyncPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            reason.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            reason: try binaryValue.field(0)
        )
    }
}
