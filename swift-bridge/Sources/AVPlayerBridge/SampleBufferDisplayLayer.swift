import AVFoundation
import Foundation

private struct AVPSampleBufferDisplayLayerPayload: Codable {
    let status: Int32
    let errorMessage: String?
    let videoGravity: String
    let readyForDisplay: Bool
    let readyForMoreMediaData: Bool
    let hasSufficientMediaDataForReliablePlaybackStart: Bool
    let requiresFlushToResumeDecoding: Bool
    let preventsCapture: Bool
    let preventsDisplaySleepDuringVideoPlayback: Bool
}

private func avpQueuedSampleBufferRenderingStatusRaw(
    _ status: AVQueuedSampleBufferRenderingStatus
) -> Int32 {
    switch status {
    case .unknown:
        return 0
    case .rendering:
        return 1
    case .failed:
        return 2
    @unknown default:
        return 0
    }
}

@_cdecl("av_sample_buffer_display_layer_create")
public func av_sample_buffer_display_layer_create() -> UnsafeMutableRawPointer? {
    avpRetained(AVSampleBufferDisplayLayer())
}

@_cdecl("av_sample_buffer_display_layer_release")
public func av_sample_buffer_display_layer_release(_ layerPtr: UnsafeMutableRawPointer?) {
    guard let layerPtr else { return }
    Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).release()
}

@_cdecl("av_sample_buffer_display_layer_info_json")
public func av_sample_buffer_display_layer_info_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    let payload = AVPSampleBufferDisplayLayerPayload(
        status: avpQueuedSampleBufferRenderingStatusRaw(layer.status),
        errorMessage: layer.error?.localizedDescription,
        videoGravity: avpVideoGravityString(layer.videoGravity),
        readyForDisplay: {
            if #available(macOS 14.4, *) {
                return layer.isReadyForDisplay
            }
            return false
        }(),
        readyForMoreMediaData: layer.isReadyForMoreMediaData,
        hasSufficientMediaDataForReliablePlaybackStart: layer.hasSufficientMediaDataForReliablePlaybackStart,
        requiresFlushToResumeDecoding: layer.requiresFlushToResumeDecoding,
        preventsCapture: layer.preventsCapture,
        preventsDisplaySleepDuringVideoPlayback: layer.preventsDisplaySleepDuringVideoPlayback
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_sample_buffer_display_layer_set_video_gravity")
public func av_sample_buffer_display_layer_set_video_gravity(
    _ layerPtr: UnsafeMutableRawPointer,
    _ gravityPtr: UnsafePointer<CChar>
) {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.videoGravity = avpVideoGravity(from: String(cString: gravityPtr))
}

@_cdecl("av_sample_buffer_display_layer_set_prevents_capture")
public func av_sample_buffer_display_layer_set_prevents_capture(
    _ layerPtr: UnsafeMutableRawPointer,
    _ preventsCapture: Bool
) {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.preventsCapture = preventsCapture
}

@_cdecl("av_sample_buffer_display_layer_set_prevents_display_sleep")
public func av_sample_buffer_display_layer_set_prevents_display_sleep(
    _ layerPtr: UnsafeMutableRawPointer,
    _ preventsDisplaySleep: Bool
) {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.preventsDisplaySleepDuringVideoPlayback = preventsDisplaySleep
}

@_cdecl("av_sample_buffer_display_layer_enqueue_sample_buffer")
public func av_sample_buffer_display_layer_enqueue_sample_buffer(
    _ layerPtr: UnsafeMutableRawPointer,
    _ sampleBufferPtr: UnsafeMutableRawPointer
) {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    let sampleBuffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBufferPtr).takeUnretainedValue()
    layer.enqueue(sampleBuffer)
}

@_cdecl("av_sample_buffer_display_layer_flush")
public func av_sample_buffer_display_layer_flush(_ layerPtr: UnsafeMutableRawPointer) {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.flush()
}

@_cdecl("av_sample_buffer_display_layer_flush_and_remove_image")
public func av_sample_buffer_display_layer_flush_and_remove_image(
    _ layerPtr: UnsafeMutableRawPointer
) {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.flushAndRemoveImage()
}

@_cdecl("av_sample_buffer_display_layer_stop_requesting_media_data")
public func av_sample_buffer_display_layer_stop_requesting_media_data(
    _ layerPtr: UnsafeMutableRawPointer
) {
    let layer = Unmanaged<AVSampleBufferDisplayLayer>.fromOpaque(layerPtr).takeUnretainedValue()
    layer.stopRequestingMediaData()
}
