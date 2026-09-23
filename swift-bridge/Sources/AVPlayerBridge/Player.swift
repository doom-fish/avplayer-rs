import AVFoundation
import AVPlayerObjCBridge
import Foundation

private final class TimeObserverBox {
    private let player: AVPlayer
    private let queue: DispatchQueue?
    private let gate: AVPCallbackGate
    private var token: Any?

    init(player: AVPlayer, queue: DispatchQueue?, gate: AVPCallbackGate, token: Any) {
        self.player = player
        self.queue = queue
        self.gate = gate
        self.token = token
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        if let token {
            player.removeTimeObserver(token)
            self.token = nil
        }
        if gate.isBusy, !avpIsCurrentQueue(queue) {
            (queue ?? DispatchQueue.main).sync {}
        }
        gate.finish()
    }
}

private struct PlayerRateDidChangeEventPayload: Codable {
    let rate: Float
    let reason: String?
    let hasOriginatingParticipant: Bool
}

@available(macOS 12.0, *)
private final class PlayerRateObserverBox: NSObject {
    private let gate: AVPCallbackGate
    private var observer: NSObjectProtocol?

    init(
        player: AVPlayer,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        gate: AVPCallbackGate
    ) {
        self.gate = gate
        super.init()
        observer = NotificationCenter.default.addObserver(
            forName: Notification.Name(rawValue: "AVPlayerRateDidChangeNotification"),
            object: player,
            queue: nil
        ) { [weak player, gate] note in
            guard let player = player ?? (note.object as? AVPlayer) else { return }
            let payload = PlayerRateDidChangeEventPayload(
                rate: player.rate,
                reason: note.userInfo?[AVPlayer.rateDidChangeReasonKey] as? String,
                hasOriginatingParticipant: note.userInfo?[AVPlayer.rateDidChangeOriginatingParticipantKey] != nil
            )
            if let queue {
                queue.async { gate.deliverJSON(payload, to: callback) }
            } else {
                gate.deliverJSON(payload, to: callback)
            }
        }
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        if let observer {
            NotificationCenter.default.removeObserver(observer)
            self.observer = nil
        }
        gate.finish()
    }
}

private struct PlayerStatusEventPayload: Codable {
    let event: String
    let status: Int32?
    let errorMessage: String?
    let timeControlStatus: Int32?
    let reasonForWaitingToPlay: String?
}

private final class PlayerStatusObserverBox: NSObject {
    private let gate: AVPCallbackGate
    private var statusObservation: NSKeyValueObservation?
    private var timeControlStatusObservation: NSKeyValueObservation?

    init(player: AVPlayer, callback: @escaping AVPJsonCallback, gate: AVPCallbackGate) {
        self.gate = gate
        super.init()
        statusObservation = player.observe(\.status, options: [.initial, .new]) { [gate] player, _ in
            gate.deliverJSON(
                PlayerStatusEventPayload(
                    event: "status_changed",
                    status: Int32(clamping: player.status.rawValue),
                    errorMessage: player.error?.localizedDescription,
                    timeControlStatus: nil,
                    reasonForWaitingToPlay: nil
                ),
                to: callback
            )
        }
        timeControlStatusObservation = player.observe(\.timeControlStatus, options: [.initial, .new]) {
            [gate] player, _ in
            gate.deliverJSON(
                PlayerStatusEventPayload(
                    event: "time_control_status_changed",
                    status: nil,
                    errorMessage: nil,
                    timeControlStatus: Int32(clamping: player.timeControlStatus.rawValue),
                    reasonForWaitingToPlay: player.reasonForWaitingToPlay?.rawValue
                ),
                to: callback
            )
        }
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        statusObservation?.invalidate()
        statusObservation = nil
        timeControlStatusObservation?.invalidate()
        timeControlStatusObservation = nil
        gate.finish()
    }
}

@_cdecl("av_player_add_status_observer")
public func av_player_add_status_observer(
    _ playerPtr: UnsafeMutableRawPointer,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing player status observer callback")
        return nil
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    return Unmanaged.passRetained(PlayerStatusObserverBox(player: player, callback: callback, gate: gate)).toOpaque()
}

@_cdecl("av_player_status_observer_release")
public func av_player_status_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    let observer = Unmanaged<PlayerStatusObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}

@_cdecl("av_player_create")
public func av_player_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    Unmanaged.passRetained(AVPlayer()).toOpaque()
}

@_cdecl("av_player_create_with_url")
public func av_player_create_with_url(
    _ urlPtr: UnsafePointer<CChar>,
    _ isFileURL: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let urlString = String(cString: urlPtr)
    let url = isFileURL ? URL(fileURLWithPath: urlString) : URL(string: urlString)
    guard let url else {
        outErrorMessage?.pointee = ffiString("invalid URL: \(urlString)")
        return nil
    }
    return Unmanaged.passRetained(AVPlayer(url: url)).toOpaque()
}

@_cdecl("av_player_create_with_asset")
public func av_player_create_with_asset(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    let player = AVPlayer(playerItem: AVPlayerItem(asset: asset, automaticallyLoadedAssetKeys: ["duration"]))
    return Unmanaged.passRetained(player).toOpaque()
}

@_cdecl("av_player_create_with_item")
public func av_player_create_with_item(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    var reason: NSString?
    guard let player = AVPTryCreatePlayerWithItem(item, &reason) else {
        outErrorMessage?.pointee = ffiString((reason as String?) ?? "AVPlayer(playerItem:) failed")
        return nil
    }
    return Unmanaged.passRetained(player).toOpaque()
}

@_cdecl("av_player_release")
public func av_player_release(_ playerPtr: UnsafeMutableRawPointer?) {
    guard let playerPtr else { return }
    Unmanaged<AVPlayer>.fromOpaque(playerPtr).release()
}

@_cdecl("av_player_info_json")
public func av_player_info_json(
    _ playerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let payload = PlayerInfoPayload(
        status: Int32(player.status.rawValue),
        errorMessage: player.error?.localizedDescription,
        rate: player.rate,
        currentTime: encodeTime(player.currentTime()),
        duration: encodeTime(player.currentItem?.duration ?? .invalid),
        timeControlStatus: Int32(player.timeControlStatus.rawValue),
        reasonForWaitingToPlay: player.reasonForWaitingToPlay?.rawValue,
        actionAtItemEnd: Int32(player.actionAtItemEnd.rawValue),
        volume: player.volume,
        muted: player.isMuted,
        automaticallyWaitsToMinimizeStalling: player.automaticallyWaitsToMinimizeStalling,
        appliesMediaSelectionCriteriaAutomatically: player.appliesMediaSelectionCriteriaAutomatically,
        eligibleForHdrPlayback: {
            if #available(macOS 10.15, *) {
                return AVPlayer.eligibleForHDRPlayback
            }
            return nil
        }(),
        audiovisualBackgroundPlaybackPolicy: {
            if #available(macOS 12.0, *) {
                return Int32(player.audiovisualBackgroundPlaybackPolicy.rawValue)
            }
            return nil
        }(),
        networkResourcePriority: {
            if #available(macOS 26.0, *) {
                return Int32(player.networkResourcePriority.rawValue)
            }
            return nil
        }(),
        defaultRate: player.defaultRate,
        audioOutputDeviceUniqueId: player.audioOutputDeviceUniqueID
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_play")
public func av_player_play(_ playerPtr: UnsafeMutableRawPointer) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.play()
}

@_cdecl("av_player_pause")
public func av_player_pause(_ playerPtr: UnsafeMutableRawPointer) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.pause()
}

@_cdecl("av_player_set_rate")
public func av_player_set_rate(_ playerPtr: UnsafeMutableRawPointer, _ rate: Float) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.rate = rate
}

@_cdecl("av_player_seek")
public func av_player_seek(
    _ playerPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    if let message = avpSeekTimeError(time, what: "seek time") {
        outErrorMessage?.pointee = ffiString(message)
        return AVP_INVALID_ARGUMENT
    }
    player.seek(to: time)
    return AVP_OK
}

@_cdecl("av_player_seek_with_tolerance")
public func av_player_seek_with_tolerance(
    _ playerPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ beforeValue: Int64,
    _ beforeTimescale: Int32,
    _ beforeKind: Int32,
    _ afterValue: Int64,
    _ afterTimescale: Int32,
    _ afterKind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    let before = cmTime(value: beforeValue, timescale: beforeTimescale, kind: beforeKind)
    let after = cmTime(value: afterValue, timescale: afterTimescale, kind: afterKind)
    if let message = avpSeekTimeError(time, what: "seek time")
        ?? avpToleranceError(before, what: "tolerance before")
        ?? avpToleranceError(after, what: "tolerance after") {
        outErrorMessage?.pointee = ffiString(message)
        return AVP_INVALID_ARGUMENT
    }
    player.seek(to: time, toleranceBefore: before, toleranceAfter: after)
    return AVP_OK
}

@_cdecl("av_player_set_rate_at_host_time")
public func av_player_set_rate_at_host_time(
    _ playerPtr: UnsafeMutableRawPointer,
    _ rate: Float,
    _ itemValue: Int64,
    _ itemTimescale: Int32,
    _ itemKind: Int32,
    _ hostValue: Int64,
    _ hostTimescale: Int32,
    _ hostKind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    guard !player.automaticallyWaitsToMinimizeStalling else {
        outErrorMessage?.pointee = ffiString(
            "setRate(_:time:atHostTime:) requires automaticallyWaitsToMinimizeStalling to be false"
        )
        return AVP_INVALID_ARGUMENT
    }
    let itemTime = cmTime(value: itemValue, timescale: itemTimescale, kind: itemKind)
    let hostTime = cmTime(value: hostValue, timescale: hostTimescale, kind: hostKind)
    var reason: NSString?
    guard AVPTrySetRateTimeAtHostTime(player, rate, itemTime, hostTime, &reason) else {
        outErrorMessage?.pointee = ffiString((reason as String?) ?? "setRate(_:time:atHostTime:) failed")
        return AVP_OPERATION_FAILED
    }
    return AVP_OK
}

@_cdecl("av_player_set_default_rate")
public func av_player_set_default_rate(_ playerPtr: UnsafeMutableRawPointer, _ rate: Float) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.defaultRate = rate
}

@_cdecl("av_player_set_audio_output_device_unique_id")
public func av_player_set_audio_output_device_unique_id(
    _ playerPtr: UnsafeMutableRawPointer,
    _ uniqueIdPtr: UnsafePointer<CChar>?
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.audioOutputDeviceUniqueID = uniqueIdPtr.map { String(cString: $0) }
}

@_cdecl("av_player_copy_current_item")
public func av_player_copy_current_item(_ playerPtr: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    guard let item = player.currentItem else { return nil }
    return Unmanaged.passRetained(item).toOpaque()
}

@_cdecl("av_player_replace_current_item")
public func av_player_replace_current_item(
    _ playerPtr: UnsafeMutableRawPointer,
    _ itemPtr: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let item = itemPtr.map { Unmanaged<AVPlayerItem>.fromOpaque($0).takeUnretainedValue() }
    var reason: NSString?
    guard AVPTryReplaceCurrentItem(player, item, &reason) else {
        outErrorMessage?.pointee = ffiString((reason as String?) ?? "replaceCurrentItem(with:) failed")
        return AVP_OPERATION_FAILED
    }
    return AVP_OK
}

@_cdecl("av_player_set_action_at_item_end")
public func av_player_set_action_at_item_end(
    _ playerPtr: UnsafeMutableRawPointer,
    _ rawValue: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    guard let value = AVPlayer.ActionAtItemEnd(rawValue: Int(rawValue)) else {
        outErrorMessage?.pointee = ffiString("invalid AVPlayerActionAtItemEnd raw value: \(rawValue)")
        return AVP_INVALID_ARGUMENT
    }
    if value == .advance, !(player is AVQueuePlayer) {
        outErrorMessage?.pointee = ffiString("AVPlayerActionAtItemEndAdvance is only supported by AVQueuePlayer")
        return AVP_INVALID_ARGUMENT
    }
    player.actionAtItemEnd = value
    return AVP_OK
}

@_cdecl("av_player_set_volume")
public func av_player_set_volume(_ playerPtr: UnsafeMutableRawPointer, _ volume: Float) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.volume = volume
}

@_cdecl("av_player_set_muted")
public func av_player_set_muted(_ playerPtr: UnsafeMutableRawPointer, _ muted: Bool) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.isMuted = muted
}

@_cdecl("av_player_set_automatically_waits_to_minimize_stalling")
public func av_player_set_automatically_waits_to_minimize_stalling(
    _ playerPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.automaticallyWaitsToMinimizeStalling = enabled
}

@_cdecl("av_player_set_applies_media_selection_criteria_automatically")
public func av_player_set_applies_media_selection_criteria_automatically(
    _ playerPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.appliesMediaSelectionCriteriaAutomatically = enabled
}

@_cdecl("av_player_set_audiovisual_background_playback_policy")
public func av_player_set_audiovisual_background_playback_policy(
    _ playerPtr: UnsafeMutableRawPointer,
    _ rawValue: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayer.audiovisualBackgroundPlaybackPolicy requires macOS 12.0+")
        return AVP_OPERATION_FAILED
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    guard let policy = AVPlayerAudiovisualBackgroundPlaybackPolicy(rawValue: Int(rawValue)) else {
        outErrorMessage?.pointee = ffiString("invalid AVPlayerAudiovisualBackgroundPlaybackPolicy raw value: \(rawValue)")
        return AVP_INVALID_ARGUMENT
    }
    player.audiovisualBackgroundPlaybackPolicy = policy
    return AVP_OK
}

@_cdecl("av_player_set_network_resource_priority")
public func av_player_set_network_resource_priority(
    _ playerPtr: UnsafeMutableRawPointer,
    _ rawValue: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayer.networkResourcePriority requires macOS 26.0+")
        return AVP_OPERATION_FAILED
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    guard let priority = AVPlayer.NetworkResourcePriority(rawValue: Int(rawValue)) else {
        outErrorMessage?.pointee = ffiString("invalid AVPlayerNetworkResourcePriority raw value: \(rawValue)")
        return AVP_INVALID_ARGUMENT
    }
    player.networkResourcePriority = priority
    return AVP_OK
}

@_cdecl("av_player_add_rate_observer")
public func av_player_add_rate_observer(
    _ playerPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerRateDidChangeNotification requires macOS 12.0+")
        return nil
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing player rate observer callback")
        return nil
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let observer = PlayerRateObserverBox(
        player: player,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_rate_observer_release")
public func av_player_rate_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    if #available(macOS 12.0, *) {
        let observer = Unmanaged<PlayerRateObserverBox>.fromOpaque(observerPtr)
        observer.takeUnretainedValue().dispose()
        observer.release()
    }
}

@_cdecl("av_player_eligible_for_hdr_playback_did_change_notification_name")
public func av_player_eligible_for_hdr_playback_did_change_notification_name(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerEligibleForHDRPlaybackDidChangeNotification requires macOS 10.15+")
        return nil
    }
    return ffiString("AVPlayerEligibleForHDRPlaybackDidChangeNotification")
}

@_cdecl("av_player_set_media_selection_criteria")
public func av_player_set_media_selection_criteria(
    _ playerPtr: UnsafeMutableRawPointer,
    _ mediaCharacteristicPtr: UnsafePointer<CChar>,
    _ criteriaPtr: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let mediaCharacteristic = avpMediaCharacteristic(from: String(cString: mediaCharacteristicPtr))
    let criteria = criteriaPtr.map {
        Unmanaged<AVPlayerMediaSelectionCriteria>.fromOpaque($0).takeUnretainedValue()
    }
    player.setMediaSelectionCriteria(criteria, forMediaCharacteristic: mediaCharacteristic)
    return AVP_OK
}

@_cdecl("av_player_copy_media_selection_criteria")
public func av_player_copy_media_selection_criteria(
    _ playerPtr: UnsafeMutableRawPointer,
    _ mediaCharacteristicPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let mediaCharacteristic = avpMediaCharacteristic(from: String(cString: mediaCharacteristicPtr))
    guard let criteria = player.mediaSelectionCriteria(forMediaCharacteristic: mediaCharacteristic) else {
        return nil
    }
    return Unmanaged.passRetained(criteria).toOpaque()
}

@_cdecl("av_player_add_periodic_time_observer")
public func av_player_add_periodic_time_observer(
    _ playerPtr: UnsafeMutableRawPointer,
    _ intervalValue: Int64,
    _ intervalTimescale: Int32,
    _ intervalKind: Int32,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPPeriodicTimeCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing periodic time callback")
        return nil
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let interval = cmTime(value: intervalValue, timescale: intervalTimescale, kind: intervalKind)
    guard interval.isNumeric, interval > .zero else {
        outErrorMessage?.pointee = ffiString("time-observer interval must be a positive numeric time")
        return nil
    }
    let queue = avpDispatchQueue(from: queueLabel)
    let token = player.addPeriodicTimeObserver(forInterval: interval, queue: queue) { [gate] time in
        gate.run(()) { context in
            let encoded = encodeTime(time)
            callback(
                context,
                encoded.value ?? 0,
                encoded.timescale ?? 0,
                kindFromEncodedTime(encoded)
            )
        }
    }
    let box = TimeObserverBox(player: player, queue: queue, gate: gate, token: token)
    return Unmanaged.passRetained(box).toOpaque()
}

@_cdecl("av_player_add_boundary_time_observer")
public func av_player_add_boundary_time_observer(
    _ playerPtr: UnsafeMutableRawPointer,
    _ timesJson: UnsafePointer<CChar>,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPSimpleCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing boundary time callback")
        return nil
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    do {
        let payloads = try avpDecodeJSON(timesJson, as: [TimePayload].self)
        let times = payloads.map { cmTime(from: $0) }
        guard !times.isEmpty else {
            outErrorMessage?.pointee = ffiString("boundary time observers need at least one time")
            return nil
        }
        guard times.allSatisfy({ $0.isNumeric }) else {
            outErrorMessage?.pointee = ffiString("boundary times must be numeric")
            return nil
        }
        let queue = avpDispatchQueue(from: queueLabel)
        let token = player.addBoundaryTimeObserver(forTimes: times.map { NSValue(time: $0) }, queue: queue) {
            [gate] in
            gate.run(()) { context in callback(context) }
        }
        let box = TimeObserverBox(player: player, queue: queue, gate: gate, token: token)
        return Unmanaged.passRetained(box).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_time_observer_release")
public func av_player_time_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    let observer = Unmanaged<TimeObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}

private func kindFromEncodedTime(_ payload: TimePayload) -> Int32 {
    switch payload.kind {
    case "numeric":
        return 0
    case "invalid":
        return 1
    case "indefinite":
        return 2
    case "positive_infinity":
        return 3
    case "negative_infinity":
        return 4
    default:
        return 1
    }
}
