import AVPlayerABI
import AVFoundation
import AVPlayerObjCBridge
import Foundation

// ── avp_asset_load_properties_async ──────────────────────────────────────────

private struct AsyncAssetPropertiesPayload: AVPBridge {
    let duration: TimePayload
    let metadata: [MetadataItemPayload]
    let isPlayable: Bool
    let isExportable: Bool
    let hasProtectedContent: Bool
    let preferredRate: Float
}

// ── avp_asset_load_properties_async ──────────────────────────────────────────

@_cdecl("avp_asset_load_properties_async")
public func avp_asset_load_properties_async(
    _ assetPtr: UnsafeMutableRawPointer,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    Task {
        do {
            async let duration = asset.load(.duration)
            async let metadata = asset.load(.metadata)
            async let isPlayable = asset.load(.isPlayable)
            async let isExportable = asset.load(.isExportable)
            async let hasProtectedContent = asset.load(.hasProtectedContent)
            async let preferredRate = asset.load(.preferredRate)

            let payload = try await AsyncAssetPropertiesPayload(
                duration: encodeTime(duration),
                metadata: metadata.map(avpEncodeMetadataItem),
                isPlayable: isPlayable,
                isExportable: isExportable,
                hasProtectedContent: hasProtectedContent,
                preferredRate: preferredRate
            )
            let value = try avpEncodeValue(payload)
            cb(value, nil, ctx)
        } catch {
            error.localizedDescription.withBridgeString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_asset_load_tracks_async ───────────────────────────────────────────────

@_cdecl("avp_asset_load_tracks_async")
public func avp_asset_load_tracks_async(
    _ assetPtr: UnsafeMutableRawPointer,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    Task {
        do {
            let tracks = try await asset.load(.tracks)
            let payloads = try await encodeTracksAsync(tracks)
            let value = try avpEncodeValue(payloads)
            cb(value, nil, ctx)
        } catch {
            error.localizedDescription.withBridgeString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_asset_load_tracks_with_media_type_async ───────────────────────────────

@_cdecl("avp_asset_load_tracks_with_media_type_async")
public func avp_asset_load_tracks_with_media_type_async(
    _ assetPtr: UnsafeMutableRawPointer,
    _ mediaTypePtr: UnsafePointer<AVPValue>,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    let mediaType = avpMediaType(from: String(bridge: mediaTypePtr))
    Task {
        do {
            let tracks = try await asset.loadTracks(withMediaType: mediaType)
            let payloads = try await encodeTracksAsync(tracks)
            let value = try avpEncodeValue(payloads)
            cb(value, nil, ctx)
        } catch {
            error.localizedDescription.withBridgeString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_asset_load_track_with_id_async ────────────────────────────────────────

@_cdecl("avp_asset_load_track_with_id_async")
public func avp_asset_load_track_with_id_async(
    _ assetPtr: UnsafeMutableRawPointer,
    _ trackID: Int32,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    Task {
        do {
            guard let track = try await asset.loadTrack(withTrackID: CMPersistentTrackID(trackID)) else {
                let value = try avpEncodeValue(AVPBinaryValue.null)
                cb(value, nil, ctx)
                return
            }
            let payload = try await encodeTrackAsync(track)
            let value = try avpEncodeValue(payload)
            cb(value, nil, ctx)
        } catch {
            error.localizedDescription.withBridgeString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_player_item_seek_async ────────────────────────────────────────────────

private typealias AVPAsyncCallback = @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void

private func avpFinishedCallback(
    _ cb: AVPAsyncCallback,
    _ ctx: UnsafeMutableRawPointer
) -> @Sendable (Bool) -> Void {
    let once = AVPOnceFlag()
    return { finished in
        guard once.claim() else { return }
        cb(try? avpEncodeValue(finished), nil, ctx)
    }
}

private func avpSeekArgumentsError(time: CMTime, before: CMTime?, after: CMTime?) -> String? {
    if let message = avpSeekTimeError(time, what: "seek time") {
        return message
    }
    if let before, let message = avpToleranceError(before, what: "tolerance before") {
        return message
    }
    if let after, let message = avpToleranceError(after, what: "tolerance after") {
        return message
    }
    return nil
}

@_cdecl("avp_player_item_seek_async")
public func avp_player_item_seek_async(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    if let message = avpSeekArgumentsError(time: time, before: nil, after: nil) {
        message.withBridgeString { cb(nil, $0, ctx) }
        return
    }
    item.seek(to: time, completionHandler: avpFinishedCallback(cb, ctx))
}

@_cdecl("avp_player_item_seek_with_tolerance_async")
public func avp_player_item_seek_with_tolerance_async(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ beforeValue: Int64,
    _ beforeTimescale: Int32,
    _ beforeKind: Int32,
    _ afterValue: Int64,
    _ afterTimescale: Int32,
    _ afterKind: Int32,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    let before = cmTime(value: beforeValue, timescale: beforeTimescale, kind: beforeKind)
    let after = cmTime(value: afterValue, timescale: afterTimescale, kind: afterKind)
    if let message = avpSeekArgumentsError(time: time, before: before, after: after) {
        message.withBridgeString { cb(nil, $0, ctx) }
        return
    }
    item.seek(
        to: time,
        toleranceBefore: before,
        toleranceAfter: after,
        completionHandler: avpFinishedCallback(cb, ctx)
    )
}

// ── avp_player_seek_async ─────────────────────────────────────────────────────

@_cdecl("avp_player_seek_async")
public func avp_player_seek_async(
    _ playerPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    if let message = avpSeekArgumentsError(time: time, before: nil, after: nil) {
        message.withBridgeString { cb(nil, $0, ctx) }
        return
    }
    player.seek(to: time, completionHandler: avpFinishedCallback(cb, ctx))
}

@_cdecl("avp_player_seek_with_tolerance_async")
public func avp_player_seek_with_tolerance_async(
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
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    let before = cmTime(value: beforeValue, timescale: beforeTimescale, kind: beforeKind)
    let after = cmTime(value: afterValue, timescale: afterTimescale, kind: afterKind)
    if let message = avpSeekArgumentsError(time: time, before: before, after: after) {
        message.withBridgeString { cb(nil, $0, ctx) }
        return
    }
    player.seek(
        to: time,
        toleranceBefore: before,
        toleranceAfter: after,
        completionHandler: avpFinishedCallback(cb, ctx)
    )
}

// ── avp_player_preroll_async ──────────────────────────────────────────────────

@_cdecl("avp_player_preroll_async")
public func avp_player_preroll_async(
    _ playerPtr: UnsafeMutableRawPointer,
    _ rate: Float,
    _ cb: @convention(c) (UnsafeMutablePointer<AVPValue>?, UnsafePointer<AVPValue>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    guard player.status == .readyToPlay else {
        "preroll requires the player status to be ReadyToPlay".withBridgeString { cb(nil, $0, ctx) }
        return
    }
    let once = AVPOnceFlag()
    var reason: NSString?
    let started = AVPTryPreroll(player, rate, { finished in
        guard once.claim() else { return }
        cb(try? avpEncodeValue(finished), nil, ctx)
    }, &reason)
    if !started, once.claim() {
        ((reason as String?) ?? "prerollAtRate(_:completionHandler:) failed").withBridgeString { cb(nil, $0, ctx) }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

private func encodeTrackAsync(_ track: AVAssetTrack) async throws -> TrackInfoPayload {
    async let naturalSize = track.load(.naturalSize)
    async let nominalFrameRate = track.load(.nominalFrameRate)
    async let estimatedDataRate = track.load(.estimatedDataRate)
    return try await TrackInfoPayload(
        trackId: track.trackID,
        mediaType: avpMediaTypeString(track.mediaType),
        naturalSize: encodeSize(naturalSize),
        nominalFrameRate: nominalFrameRate,
        estimatedDataRate: estimatedDataRate
    )
}

private func encodeTracksAsync(_ tracks: [AVAssetTrack]) async throws -> [TrackInfoPayload] {
    var payloads: [TrackInfoPayload] = []
    payloads.reserveCapacity(tracks.count)
    for track in tracks {
        payloads.append(try await encodeTrackAsync(track))
    }
    return payloads
}

extension AsyncAssetPropertiesPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            duration.binaryValue,
            metadata.binaryValue,
            isPlayable.binaryValue,
            isExportable.binaryValue,
            hasProtectedContent.binaryValue,
            preferredRate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            duration: try binaryValue.field(0),
            metadata: try binaryValue.field(1),
            isPlayable: try binaryValue.field(2),
            isExportable: try binaryValue.field(3),
            hasProtectedContent: try binaryValue.field(4),
            preferredRate: try binaryValue.field(5)
        )
    }
}
