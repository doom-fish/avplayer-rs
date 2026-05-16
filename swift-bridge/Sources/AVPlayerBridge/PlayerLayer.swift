import AVFoundation
import Foundation
import QuartzCore

@_cdecl("av_player_layer_create")
public func av_player_layer_create(
    _ playerPtr: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer? {
    let player = playerPtr.map { Unmanaged<AVPlayer>.fromOpaque($0).takeUnretainedValue() }
    return Unmanaged.passRetained(AVPlayerLayer(player: player)).toOpaque()
}

@_cdecl("av_player_layer_release")
public func av_player_layer_release(_ layerPtr: UnsafeMutableRawPointer?) {
    guard let layerPtr else { return }
    Unmanaged<AVPlayerLayer>.fromOpaque(layerPtr).release()
}

@_cdecl("av_player_layer_info_json")
public func av_player_layer_info_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let layer = Unmanaged<AVPlayerLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    let payload = PlayerLayerInfoPayload(
        hasPlayer: layer.player != nil,
        videoGravity: avpVideoGravityString(layer.videoGravity),
        readyForDisplay: layer.isReadyForDisplay,
        videoRect: encodeRect(layer.videoRect)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_layer_set_player")
public func av_player_layer_set_player(
    _ layerPtr: UnsafeMutableRawPointer,
    _ playerPtr: UnsafeMutableRawPointer?
) {
    let layer = Unmanaged<AVPlayerLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.player = playerPtr.map { Unmanaged<AVPlayer>.fromOpaque($0).takeUnretainedValue() }
}

@_cdecl("av_player_layer_set_video_gravity")
public func av_player_layer_set_video_gravity(
    _ layerPtr: UnsafeMutableRawPointer,
    _ gravityPtr: UnsafePointer<CChar>
) {
    let layer = Unmanaged<AVPlayerLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.videoGravity = avpVideoGravity(from: String(cString: gravityPtr))
}

@_cdecl("av_player_layer_copy_displayed_pixel_buffer")
public func av_player_layer_copy_displayed_pixel_buffer(
    _ layerPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let layer = Unmanaged<AVPlayerLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    guard let pixelBuffer = layer.displayedPixelBuffer() else { return nil }
    return Unmanaged.passRetained(pixelBuffer).toOpaque()
}
