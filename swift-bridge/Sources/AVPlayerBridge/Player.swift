import AVFoundation
import Foundation

private final class TimeObserverBox {
    private let player: AVPlayer
    private let token: Any
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        player: AVPlayer,
        token: Any,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.player = player
        self.token = token
        self.userData = userData
        self.dropUserData = dropUserData
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        player.removeTimeObserver(token)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }
}

private struct PlayerRateDidChangeEventPayload: Codable {
    let rate: Float
    let reason: String?
    let hasOriginatingParticipant: Bool
}

@available(macOS 12.0, *)
private final class PlayerRateObserverBox: NSObject {
    private weak var player: AVPlayer?
    private let callback: AVPJsonCallback
    private let queue: DispatchQueue?
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private let deliveryGroup = DispatchGroup()
    private var observer: NSObjectProtocol?
    private var disposed = false

    init(
        player: AVPlayer,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.player = player
        self.callback = callback
        self.queue = queue
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        observer = NotificationCenter.default.addObserver(
            forName: Notification.Name(rawValue: "AVPlayerRateDidChangeNotification"),
            object: player,
            queue: nil
        ) { [weak self, weak player] note in
            guard let self, let player = player ?? (note.object as? AVPlayer) else { return }
            self.send(
                PlayerRateDidChangeEventPayload(
                    rate: player.rate,
                    reason: note.userInfo?[AVPlayer.rateDidChangeReasonKey] as? String,
                    hasOriginatingParticipant: note.userInfo?[AVPlayer.rateDidChangeOriginatingParticipantKey] != nil
                )
            )
        }
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        if let observer {
            NotificationCenter.default.removeObserver(observer)
            self.observer = nil
        }
        deliveryGroup.wait()
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    private func send(_ payload: PlayerRateDidChangeEventPayload) {
        let deliver = { [callback, userData] in
            guard let json = try? avpEncodeJSON(payload) else {
                callback(userData, nil)
                return
            }
            json.withCString { callback(userData, $0) }
        }
        if let queue {
            deliveryGroup.enter()
            queue.async { [self] in
                defer { deliveryGroup.leave() }
                deliver()
            }
        } else {
            deliver()
        }
    }
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
    return Unmanaged.passRetained(AVPlayer(playerItem: item)).toOpaque()
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
        }()
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
    guard time != .invalid else {
        outErrorMessage?.pointee = ffiString("seek time must be numeric")
        return AVP_INVALID_ARGUMENT
    }
    player.seek(to: time)
    return AVP_OK
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
    _ itemPtr: UnsafeMutableRawPointer?
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let item = itemPtr.map { Unmanaged<AVPlayerItem>.fromOpaque($0).takeUnretainedValue() }
    player.replaceCurrentItem(with: item)
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
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_rate_observer_release")
public func av_player_rate_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    if #available(macOS 12.0, *) {
        Unmanaged<PlayerRateObserverBox>.fromOpaque(observerPtr).release()
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
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing periodic time callback")
        return nil
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let interval = cmTime(value: intervalValue, timescale: intervalTimescale, kind: intervalKind)
    guard interval != .invalid, interval != .indefinite else {
        outErrorMessage?.pointee = ffiString("time-observer interval must be numeric")
        return nil
    }
    let queue = avpDispatchQueue(from: queueLabel)
    let token = player.addPeriodicTimeObserver(forInterval: interval, queue: queue) { time in
        let encoded = encodeTime(time)
        callback(
            userData,
            encoded.value ?? 0,
            encoded.timescale ?? 0,
            kindFromEncodedTime(encoded)
        )
    }
    let box = TimeObserverBox(player: player, token: token, userData: userData, dropUserData: dropUserData)
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
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing boundary time callback")
        return nil
    }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    do {
        let payloads = try avpDecodeJSON(timesJson, as: [TimePayload].self)
        let times = payloads.map { NSValue(time: cmTime(from: $0)) }
        let queue = avpDispatchQueue(from: queueLabel)
        let token = player.addBoundaryTimeObserver(forTimes: times, queue: queue) {
            callback(userData)
        }
        let box = TimeObserverBox(
            player: player,
            token: token,
            userData: userData,
            dropUserData: dropUserData
        )
        return Unmanaged.passRetained(box).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_time_observer_release")
public func av_player_time_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<TimeObserverBox>.fromOpaque(observerPtr).release()
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
