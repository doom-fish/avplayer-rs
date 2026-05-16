import AVFoundation
import Foundation

public typealias AVPJsonCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?
) -> Void
public typealias AVPPeriodicTimeCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int64,
    Int32,
    Int32
) -> Void
public typealias AVPSimpleCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias AVPDropCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

private final class PlayerItemObserverBox: NSObject {
    private let item: AVPlayerItem
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false
    private var statusObservation: NSKeyValueObservation?
    private var presentationSizeObservation: NSKeyValueObservation?
    private var endObserver: NSObjectProtocol?

    init(
        item: AVPlayerItem,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.item = item
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        registerObservers()
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        statusObservation?.invalidate()
        presentationSizeObservation?.invalidate()
        statusObservation = nil
        presentationSizeObservation = nil
        if let endObserver {
            NotificationCenter.default.removeObserver(endObserver)
            self.endObserver = nil
        }
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    private func registerObservers() {
        statusObservation = item.observe(\.status, options: [.initial, .new]) { [weak self] item, _ in
            guard let self else { return }
            self.send(
                PlayerItemEventPayload(
                    event: "status_changed",
                    status: Int32(item.status.rawValue),
                    errorMessage: item.error?.localizedDescription,
                    presentationSize: nil
                )
            )
        }

        presentationSizeObservation = item.observe(\.presentationSize, options: [.initial, .new]) {
            [weak self] item, _ in
            guard let self else { return }
            self.send(
                PlayerItemEventPayload(
                    event: "presentation_size_changed",
                    status: nil,
                    errorMessage: nil,
                    presentationSize: encodeSize(item.presentationSize)
                )
            )
        }

        endObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime,
            object: item,
            queue: nil
        ) { [weak self] _ in
            self?.send(
                PlayerItemEventPayload(
                    event: "did_play_to_end",
                    status: nil,
                    errorMessage: nil,
                    presentationSize: nil
                )
            )
        }
    }

    private func send(_ payload: PlayerItemEventPayload) {
        guard !disposed else { return }
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
    }
}

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

@_cdecl("av_player_item_create_with_url")
public func av_player_item_create_with_url(
    _ urlPtr: UnsafePointer<CChar>,
    _ isFileURL: Bool,
    _ assetKeysJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let urlString = String(cString: urlPtr)
    let url = isFileURL ? URL(fileURLWithPath: urlString) : URL(string: urlString)
    guard let url else {
        outErrorMessage?.pointee = ffiString("invalid URL: \(urlString)")
        return nil
    }

    do {
        let assetKeys = try avpDecodeJSON(assetKeysJson, as: [String].self)
        let asset = AVURLAsset(url: url)
        let item = AVPlayerItem(asset: asset, automaticallyLoadedAssetKeys: assetKeys)
        return Unmanaged.passRetained(item).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_create_with_asset")
public func av_player_item_create_with_asset(
    _ assetPtr: UnsafeMutableRawPointer,
    _ assetKeysJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    do {
        let assetKeys = try avpDecodeJSON(assetKeysJson, as: [String].self)
        let item = AVPlayerItem(asset: asset, automaticallyLoadedAssetKeys: assetKeys)
        return Unmanaged.passRetained(item).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_release")
public func av_player_item_release(_ itemPtr: UnsafeMutableRawPointer?) {
    guard let itemPtr else { return }
    Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).release()
}

@_cdecl("av_player_item_info_json")
public func av_player_item_info_json(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let payload = PlayerItemInfoPayload(
        status: Int32(item.status.rawValue),
        errorMessage: item.error?.localizedDescription,
        duration: encodeTime(item.duration),
        presentationSize: encodeSize(item.presentationSize),
        metadata: item.asset.metadata.map(avpEncodeMetadataItem)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_add_observer")
public func av_player_item_add_observer(
    _ itemPtr: UnsafeMutableRawPointer,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing player-item observer callback")
        return nil
    }
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let box = PlayerItemObserverBox(
        item: item,
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(box).toOpaque()
}

@_cdecl("av_player_item_observer_release")
public func av_player_item_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<PlayerItemObserverBox>.fromOpaque(observerPtr).release()
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
        duration: encodeTime(player.currentItem?.duration ?? .invalid)
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
