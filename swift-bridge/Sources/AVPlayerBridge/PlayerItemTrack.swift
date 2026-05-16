import AVFoundation
import Foundation

@_cdecl("av_player_item_track_release")
public func av_player_item_track_release(_ trackPtr: UnsafeMutableRawPointer?) {
    guard let trackPtr else { return }
    Unmanaged<AVPlayerItemTrack>.fromOpaque(trackPtr).release()
}

@_cdecl("av_player_item_track_info_json")
public func av_player_item_track_info_json(
    _ trackPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let track = Unmanaged<AVPlayerItemTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    let payload = PlayerItemTrackInfoPayload(
        enabled: track.isEnabled,
        currentVideoFrameRate: track.currentVideoFrameRate,
        videoFieldMode: track.videoFieldMode,
        hasAssetTrack: track.assetTrack != nil
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_track_set_enabled")
public func av_player_item_track_set_enabled(
    _ trackPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let track = Unmanaged<AVPlayerItemTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    track.isEnabled = enabled
}

@_cdecl("av_player_item_track_set_video_field_mode")
public func av_player_item_track_set_video_field_mode(
    _ trackPtr: UnsafeMutableRawPointer,
    _ modePtr: UnsafePointer<CChar>?
) {
    let track = Unmanaged<AVPlayerItemTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    track.videoFieldMode = modePtr.map { String(cString: $0) }
}

@_cdecl("av_player_item_track_copy_asset_track")
public func av_player_item_track_copy_asset_track(
    _ trackPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let track = Unmanaged<AVPlayerItemTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    guard let assetTrack = track.assetTrack else { return nil }
    return Unmanaged.passRetained(assetTrack).toOpaque()
}
