import AVFoundation
import Foundation

// ── avp_asset_load_properties_async ──────────────────────────────────────────

private struct AsyncAssetPropertiesPayload: Codable {
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
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
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
            let json = try avpEncodeJSON(payload)
            guard let cstr = ffiString(json) else {
                "failed to allocate JSON string".withCString { cb(nil, $0, ctx) }
                return
            }
            cb(UnsafeRawPointer(cstr), nil, ctx)
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_asset_load_tracks_async ───────────────────────────────────────────────

@_cdecl("avp_asset_load_tracks_async")
public func avp_asset_load_tracks_async(
    _ assetPtr: UnsafeMutableRawPointer,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    Task {
        do {
            let tracks = try await asset.load(.tracks)
            let payloads = try await encodeTracksAsync(tracks)
            let json = try avpEncodeJSON(payloads)
            guard let cstr = ffiString(json) else {
                "failed to allocate JSON string".withCString { cb(nil, $0, ctx) }
                return
            }
            cb(UnsafeRawPointer(cstr), nil, ctx)
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_asset_load_tracks_with_media_type_async ───────────────────────────────

@_cdecl("avp_asset_load_tracks_with_media_type_async")
public func avp_asset_load_tracks_with_media_type_async(
    _ assetPtr: UnsafeMutableRawPointer,
    _ mediaTypePtr: UnsafePointer<CChar>,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    let mediaType = AVMediaType(rawValue: String(cString: mediaTypePtr))
    Task {
        do {
            let tracks = try await asset.loadTracks(withMediaType: mediaType)
            let payloads = try await encodeTracksAsync(tracks)
            let json = try avpEncodeJSON(payloads)
            guard let cstr = ffiString(json) else {
                "failed to allocate JSON string".withCString { cb(nil, $0, ctx) }
                return
            }
            cb(UnsafeRawPointer(cstr), nil, ctx)
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_asset_load_track_with_id_async ────────────────────────────────────────

@_cdecl("avp_asset_load_track_with_id_async")
public func avp_asset_load_track_with_id_async(
    _ assetPtr: UnsafeMutableRawPointer,
    _ trackID: Int32,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    Task {
        do {
            guard let track = try await asset.loadTrack(withTrackID: CMPersistentTrackID(trackID)) else {
                // No track with that ID — return JSON null
                guard let cstr = ffiString("null") else {
                    "failed to allocate null payload".withCString { cb(nil, $0, ctx) }
                    return
                }
                cb(UnsafeRawPointer(cstr), nil, ctx)
                return
            }
            let payload = try await encodeTrackAsync(track)
            let json = try avpEncodeJSON(payload)
            guard let cstr = ffiString(json) else {
                "failed to allocate JSON string".withCString { cb(nil, $0, ctx) }
                return
            }
            cb(UnsafeRawPointer(cstr), nil, ctx)
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    }
}

// ── avp_player_item_seek_async ────────────────────────────────────────────────

@_cdecl("avp_player_item_seek_async")
public func avp_player_item_seek_async(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    Task {
        let finished = await withCheckedContinuation { (cont: CheckedContinuation<Bool, Never>) in
            item.seek(to: time) { finished in cont.resume(returning: finished) }
        }
        cb(UnsafeRawPointer(bitPattern: finished ? 1 : 0), nil, ctx)
    }
}

// ── avp_player_seek_async ─────────────────────────────────────────────────────

@_cdecl("avp_player_seek_async")
public func avp_player_seek_async(
    _ playerPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let time = cmTime(value: value, timescale: timescale, kind: kind)
    Task {
        let finished = await withCheckedContinuation { (cont: CheckedContinuation<Bool, Never>) in
            player.seek(to: time) { finished in cont.resume(returning: finished) }
        }
        cb(UnsafeRawPointer(bitPattern: finished ? 1 : 0), nil, ctx)
    }
}

// ── avp_player_preroll_async ──────────────────────────────────────────────────

@_cdecl("avp_player_preroll_async")
public func avp_player_preroll_async(
    _ playerPtr: UnsafeMutableRawPointer,
    _ rate: Float,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    Task {
        let finished = await withCheckedContinuation { (cont: CheckedContinuation<Bool, Never>) in
            player.preroll(atRate: rate) { finished in cont.resume(returning: finished) }
        }
        cb(UnsafeRawPointer(bitPattern: finished ? 1 : 0), nil, ctx)
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
        nominalFrameRate: String(nominalFrameRate),
        estimatedDataRate: String(estimatedDataRate)
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
