import AVFoundation
import Foundation

private func avpAssetPlaybackConfigurationOptionString(
    _ option: AVAssetPlaybackConfigurationOption
) -> String {
    if option == .stereoVideo {
        return "stereo_video"
    }
    if option == .stereoMultiviewVideo {
        return "stereo_multiview_video"
    }
    if #available(macOS 15.0, *), option == .spatialVideo {
        return "spatial_video"
    }
    if #available(macOS 26.0, *) {
        if option == .nonRectilinearProjection {
            return "non_rectilinear_projection"
        }
        if option == .appleImmersiveVideo {
            return "apple_immersive_video"
        }
    }
    return option.rawValue
}

@_cdecl("av_asset_playback_assistant_create")
public func av_asset_playback_assistant_create(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetPlaybackAssistant requires macOS 13.0")
        return nil
    }
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return Unmanaged.passRetained(AVAssetPlaybackAssistant(asset: asset)).toOpaque()
}

@_cdecl("av_asset_playback_assistant_copy_options_json")
public func av_asset_playback_assistant_copy_options_json(
    _ assistantPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetPlaybackAssistant requires macOS 13.0")
        return nil
    }
    let assistant = Unmanaged<AVAssetPlaybackAssistant>.fromOpaque(assistantPtr).takeUnretainedValue()
    let semaphore = DispatchSemaphore(value: 0)
    var encoded: UnsafeMutablePointer<CChar>?
    assistant.loadPlaybackConfigurationOptions { options in
        defer { semaphore.signal() }
        let values = options.map(avpAssetPlaybackConfigurationOptionString)
        encoded = ffiString((try? avpEncodeJSON(values)) ?? "[]")
    }
    if semaphore.wait(timeout: .now() + .seconds(30)) == .timedOut {
        outErrorMessage?.pointee = ffiString("timed out waiting for playback configuration options")
        return nil
    }
    return encoded
}
