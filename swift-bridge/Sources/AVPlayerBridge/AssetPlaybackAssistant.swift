import AVPlayerABI
import AVFoundation
import Foundation

@available(macOS 13.0, *)
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
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetPlaybackAssistant requires macOS 13.0")
        return nil
    }
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return Unmanaged.passRetained(AVAssetPlaybackAssistant(asset: asset)).toOpaque()
}

@_cdecl("av_asset_playback_assistant_copy_options_value")
public func av_asset_playback_assistant_copy_options_value(
    _ assistantPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetPlaybackAssistant requires macOS 13.0")
        return nil
    }
    let assistant = Unmanaged<AVAssetPlaybackAssistant>.fromOpaque(assistantPtr).takeUnretainedValue()
    switch avpAwait(label: "playback configuration options", work: {
        await withCheckedContinuation { (continuation: CheckedContinuation<[String], Never>) in
            assistant.loadPlaybackConfigurationOptions { options in
                continuation.resume(returning: options.map(avpAssetPlaybackConfigurationOptionString))
            }
        }
    }) {
    case .success(let values):
        do {
            return try avpEncodeValue(values)
        } catch {
            outErrorMessage?.pointee = ffiString(error.localizedDescription)
            return nil
        }
    case .failure(let error):
        avpWriteError(error, outErrorMessage)
        return nil
    }
}
