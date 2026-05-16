import AVFoundation
import Foundation

private final class PlayerItemObserverBox: NSObject {
    private let item: AVPlayerItem
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false
    private var statusObservation: NSKeyValueObservation?
    private var presentationSizeObservation: NSKeyValueObservation?
    private var endObserver: NSObjectProtocol?
    private var stalledObserver: NSObjectProtocol?
    private var accessLogObserver: NSObjectProtocol?
    private var errorLogObserver: NSObjectProtocol?
    private var mediaSelectionObserver: NSObjectProtocol?

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
        [endObserver, stalledObserver, accessLogObserver, errorLogObserver, mediaSelectionObserver]
            .compactMap { $0 }
            .forEach(NotificationCenter.default.removeObserver)
        endObserver = nil
        stalledObserver = nil
        accessLogObserver = nil
        errorLogObserver = nil
        mediaSelectionObserver = nil
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
            self?.send(PlayerItemEventPayload(event: "did_play_to_end", status: nil, errorMessage: nil, presentationSize: nil))
        }

        stalledObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemPlaybackStalled,
            object: item,
            queue: nil
        ) { [weak self] _ in
            self?.send(PlayerItemEventPayload(event: "playback_stalled", status: nil, errorMessage: nil, presentationSize: nil))
        }

        accessLogObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemNewAccessLogEntry,
            object: item,
            queue: nil
        ) { [weak self] _ in
            self?.send(PlayerItemEventPayload(event: "new_access_log_entry", status: nil, errorMessage: nil, presentationSize: nil))
        }

        errorLogObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemNewErrorLogEntry,
            object: item,
            queue: nil
        ) { [weak self] _ in
            self?.send(PlayerItemEventPayload(event: "new_error_log_entry", status: nil, errorMessage: nil, presentationSize: nil))
        }

        mediaSelectionObserver = NotificationCenter.default.addObserver(
            forName: Notification.Name(rawValue: "AVPlayerItemMediaSelectionDidChangeNotification"),
            object: item,
            queue: nil
        ) { [weak self] _ in
            self?.send(PlayerItemEventPayload(event: "media_selection_changed", status: nil, errorMessage: nil, presentationSize: nil))
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
        metadata: item.asset.metadata.map(avpEncodeMetadataItem),
        automaticallyLoadedAssetKeys: item.automaticallyLoadedAssetKeys,
        seekableTimeRanges: encodeTimeRanges(item.seekableTimeRanges),
        loadedTimeRanges: encodeTimeRanges(item.loadedTimeRanges),
        canUseNetworkResourcesForLiveStreamingWhilePaused: item.canUseNetworkResourcesForLiveStreamingWhilePaused,
        preferredForwardBufferDuration: item.preferredForwardBufferDuration,
        preferredPeakBitRate: item.preferredPeakBitRate,
        preferredPeakBitRateForExpensiveNetworks: item.preferredPeakBitRateForExpensiveNetworks,
        preferredMaximumResolution: encodeSize(item.preferredMaximumResolution),
        preferredMaximumResolutionForExpensiveNetworks: encodeSize(item.preferredMaximumResolutionForExpensiveNetworks),
        audioTimePitchAlgorithm: avpAudioTimePitchAlgorithmString(item.audioTimePitchAlgorithm),
        outputCount: item.outputs.count,
        trackCount: item.tracks.count
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

@_cdecl("av_player_item_set_can_use_network_resources_for_live_streaming_while_paused")
public func av_player_item_set_can_use_network_resources_for_live_streaming_while_paused(
    _ itemPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.canUseNetworkResourcesForLiveStreamingWhilePaused = enabled
}

@_cdecl("av_player_item_set_preferred_forward_buffer_duration")
public func av_player_item_set_preferred_forward_buffer_duration(
    _ itemPtr: UnsafeMutableRawPointer,
    _ duration: Double
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.preferredForwardBufferDuration = duration
}

@_cdecl("av_player_item_set_preferred_peak_bit_rate")
public func av_player_item_set_preferred_peak_bit_rate(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Double
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.preferredPeakBitRate = value
}

@_cdecl("av_player_item_set_preferred_peak_bit_rate_for_expensive_networks")
public func av_player_item_set_preferred_peak_bit_rate_for_expensive_networks(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Double
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.preferredPeakBitRateForExpensiveNetworks = value
}

@_cdecl("av_player_item_set_preferred_maximum_resolution")
public func av_player_item_set_preferred_maximum_resolution(
    _ itemPtr: UnsafeMutableRawPointer,
    _ width: Double,
    _ height: Double
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.preferredMaximumResolution = CGSize(width: width, height: height)
}

@_cdecl("av_player_item_set_preferred_maximum_resolution_for_expensive_networks")
public func av_player_item_set_preferred_maximum_resolution_for_expensive_networks(
    _ itemPtr: UnsafeMutableRawPointer,
    _ width: Double,
    _ height: Double
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.preferredMaximumResolutionForExpensiveNetworks = CGSize(width: width, height: height)
}

@_cdecl("av_player_item_set_audio_time_pitch_algorithm")
public func av_player_item_set_audio_time_pitch_algorithm(
    _ itemPtr: UnsafeMutableRawPointer,
    _ algorithmPtr: UnsafePointer<CChar>
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.audioTimePitchAlgorithm = avpAudioTimePitchAlgorithm(from: String(cString: algorithmPtr))
}

@_cdecl("av_player_item_track_count")
public func av_player_item_track_count(_ itemPtr: UnsafeMutableRawPointer) -> Int32 {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    return Int32(item.tracks.count)
}

@_cdecl("av_player_item_copy_track_at_index")
public func av_player_item_copy_track_at_index(
    _ itemPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    guard index >= 0, Int(index) < item.tracks.count else { return nil }
    return Unmanaged.passRetained(item.tracks[Int(index)]).toOpaque()
}

@_cdecl("av_player_item_copy_access_log")
public func av_player_item_copy_access_log(_ itemPtr: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    guard let log = item.accessLog() else { return nil }
    return Unmanaged.passRetained(log).toOpaque()
}

@_cdecl("av_player_item_copy_error_log")
public func av_player_item_copy_error_log(_ itemPtr: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    guard let log = item.errorLog() else { return nil }
    return Unmanaged.passRetained(log).toOpaque()
}

@_cdecl("av_player_item_add_output")
public func av_player_item_add_output(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let outputBox = Unmanaged<AVPPlayerItemOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    item.add(outputBox.output)
    return AVP_OK
}

@_cdecl("av_player_item_remove_output")
public func av_player_item_remove_output(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let outputBox = Unmanaged<AVPPlayerItemOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    item.remove(outputBox.output)
}
