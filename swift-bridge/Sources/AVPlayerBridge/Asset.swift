import AVFoundation
import Foundation

@_cdecl("av_asset_release")
public func av_asset_release(_ assetPtr: UnsafeMutableRawPointer?) {
    guard let assetPtr else { return }
    Unmanaged<AVAsset>.fromOpaque(assetPtr).release()
}

@_cdecl("av_asset_info_json")
public func av_asset_info_json(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    switch avpAwait(label: "AVAsset duration and metadata", work: { try await makeAssetInfo(asset: asset) }) {
    case .success(let payload):
        guard let json = try? avpEncodeJSON(payload) else {
            outErrorMessage?.pointee = ffiString("failed to encode asset info payload")
            return nil
        }
        return ffiString(json)
    case .failure(let error):
        avpWriteError(error, outErrorMessage)
        return nil
    }
}

@_cdecl("av_asset_load_values_json")
public func av_asset_load_values_json(
    _ assetPtr: UnsafeMutableRawPointer,
    _ keysJson: UnsafePointer<CChar>,
    _ timeoutSeconds: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    do {
        let keys = try avpDecodeJSON(keysJson, as: [String].self)
        let semaphore = DispatchSemaphore(value: 0)
        asset.loadValuesAsynchronously(forKeys: keys) {
            semaphore.signal()
        }
        let timeout = max(Int(timeoutSeconds), 1)
        if semaphore.wait(timeout: .now() + .seconds(timeout)) == .timedOut {
            outErrorMessage?.pointee = ffiString("timed out waiting for AVAsset key loading")
            return nil
        }

        let statuses = keys.map { key -> KeyLoadStatusPayload in
            var error: NSError?
            let status = asset.statusOfValue(forKey: key, error: &error)
            return KeyLoadStatusPayload(
                key: key,
                status: Int32(status.rawValue),
                errorMessage: error?.localizedDescription
            )
        }
        return ffiString(try avpEncodeJSON(statuses))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_status_of_value")
public func av_asset_status_of_value(
    _ assetPtr: UnsafeMutableRawPointer,
    _ keyPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    let key = String(cString: keyPtr)
    var error: NSError?
    let status = asset.statusOfValue(forKey: key, error: &error)
    if let error, status == .failed {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
    }
    return Int32(status.rawValue)
}

@_cdecl("av_asset_load_tracks")
public func av_asset_load_tracks(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    switch avpAwait(label: "AVAsset tracks", work: { try await asset.load(.tracks) }) {
    case .success(let tracks):
        outStatus?.pointee = AVP_OK
        return Unmanaged.passRetained(AVPObjectArrayBox(objects: tracks)).toOpaque()
    case .failure(let error):
        outStatus?.pointee = error.status
        avpWriteError(error, outErrorMessage)
        return nil
    }
}

@_cdecl("av_asset_track_release")
public func av_asset_track_release(_ trackPtr: UnsafeMutableRawPointer?) {
    guard let trackPtr else { return }
    Unmanaged<AVAssetTrack>.fromOpaque(trackPtr).release()
}

@_cdecl("av_asset_track_info_json")
public func av_asset_track_info_json(
    _ trackPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let track = Unmanaged<AVAssetTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    switch avpAwait(label: "AVAssetTrack properties", work: { try await makeTrackInfo(track: track) }) {
    case .success(let payload):
        guard let json = try? avpEncodeJSON(payload) else {
            outErrorMessage?.pointee = ffiString("failed to encode track info payload")
            return nil
        }
        return ffiString(json)
    case .failure(let error):
        avpWriteError(error, outErrorMessage)
        return nil
    }
}

private func makeAssetInfo(asset: AVAsset) async throws -> AssetInfoPayload {
    let duration = try await asset.load(.duration)
    let metadata = try await asset.load(.metadata)
    return AssetInfoPayload(
        url: (asset as? AVURLAsset)?.url.absoluteString,
        duration: encodeTime(duration),
        metadata: metadata.map(avpEncodeMetadataItem)
    )
}

private func makeTrackInfo(track: AVAssetTrack) async throws -> TrackInfoPayload {
    let naturalSize = try await track.load(.naturalSize)
    let nominalFrameRate = try await track.load(.nominalFrameRate)
    let estimatedDataRate = try await track.load(.estimatedDataRate)
    return TrackInfoPayload(
        trackId: track.trackID,
        mediaType: avpMediaTypeString(track.mediaType),
        naturalSize: encodeSize(naturalSize),
        nominalFrameRate: String(nominalFrameRate),
        estimatedDataRate: String(estimatedDataRate)
    )
}
