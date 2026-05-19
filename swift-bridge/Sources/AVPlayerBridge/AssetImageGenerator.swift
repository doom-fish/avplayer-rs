import AVFoundation
import CoreGraphics
import Foundation

private struct AssetImageGeneratorInfoPayload: Codable {
    let appliesPreferredTrackTransform: Bool
    let maximumSize: SizePayload
    let apertureMode: String?
    let requestedTimeToleranceBefore: TimePayload
    let requestedTimeToleranceAfter: TimePayload
    let dynamicRangePolicy: String?
    let hasCustomVideoCompositor: Bool
    let customVideoCompositorClassName: String?
}

private struct AssetImagePayload: Codable {
    let width: Int
    let height: Int
    let bitsPerComponent: Int
    let bitsPerPixel: Int
    let bytesPerRow: Int
    let alphaInfo: UInt32
    let bitmapInfo: UInt32
    let renderingIntent: UInt32
}

private final class AVPAssetImageBox: NSObject {
    let image: CGImage

    init(image: CGImage) {
        self.image = image
    }
}

private func avpAssetImageGeneratorApertureModeString(
    _ mode: AVAssetImageGenerator.ApertureMode?
) -> String? {
    guard let mode else { return nil }
    switch mode {
    case .cleanAperture:
        return "clean_aperture"
    case .productionAperture:
        return "production_aperture"
    case .encodedPixels:
        return "encoded_pixels"
    default:
        return mode.rawValue
    }
}

private func avpAssetImageGeneratorApertureMode(
    from raw: String
) -> AVAssetImageGenerator.ApertureMode {
    switch raw {
    case "clean_aperture":
        return .cleanAperture
    case "production_aperture":
        return .productionAperture
    case "encoded_pixels":
        return .encodedPixels
    default:
        return AVAssetImageGenerator.ApertureMode(rawValue: raw)
    }
}

@available(macOS 15.0, *)
private func avpAssetImageGeneratorDynamicRangePolicyString(
    _ policy: AVAssetImageGenerator.DynamicRangePolicy?
) -> String? {
    guard let policy else { return nil }
    switch policy {
    case .forceSDR:
        return "force_sdr"
    case .matchSource:
        return "match_source"
    default:
        return policy.rawValue
    }
}

@available(macOS 15.0, *)
private func avpAssetImageGeneratorDynamicRangePolicy(
    from raw: String
) -> AVAssetImageGenerator.DynamicRangePolicy {
    switch raw {
    case "force_sdr":
        return .forceSDR
    case "match_source":
        return .matchSource
    default:
        return AVAssetImageGenerator.DynamicRangePolicy(rawValue: raw)
    }
}

private func makeAssetImageGeneratorInfoPayload(
    _ generator: AVAssetImageGenerator
) -> AssetImageGeneratorInfoPayload {
    AssetImageGeneratorInfoPayload(
        appliesPreferredTrackTransform: generator.appliesPreferredTrackTransform,
        maximumSize: encodeSize(generator.maximumSize),
        apertureMode: avpAssetImageGeneratorApertureModeString(generator.apertureMode),
        requestedTimeToleranceBefore: encodeTime(generator.requestedTimeToleranceBefore),
        requestedTimeToleranceAfter: encodeTime(generator.requestedTimeToleranceAfter),
        dynamicRangePolicy: {
            if #available(macOS 15.0, *) {
                return avpAssetImageGeneratorDynamicRangePolicyString(generator.dynamicRangePolicy)
            }
            return nil
        }(),
        hasCustomVideoCompositor: generator.customVideoCompositor != nil,
        customVideoCompositorClassName: generator.customVideoCompositor.map {
            String(describing: type(of: $0))
        }
    )
}

private func makeAssetImagePayload(_ image: CGImage) -> AssetImagePayload {
    AssetImagePayload(
        width: image.width,
        height: image.height,
        bitsPerComponent: image.bitsPerComponent,
        bitsPerPixel: image.bitsPerPixel,
        bytesPerRow: image.bytesPerRow,
        alphaInfo: UInt32(image.alphaInfo.rawValue),
        bitmapInfo: image.bitmapInfo.rawValue,
        renderingIntent: UInt32(image.renderingIntent.rawValue)
    )
}

@_cdecl("av_asset_image_generator_create")
public func av_asset_image_generator_create(
    _ assetPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return Unmanaged.passRetained(AVAssetImageGenerator(asset: asset)).toOpaque()
}

@_cdecl("av_asset_image_generator_info_json")
public func av_asset_image_generator_info_json(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    do {
        return ffiString(try avpEncodeJSON(makeAssetImageGeneratorInfoPayload(generator)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_image_generator_set_applies_preferred_track_transform")
public func av_asset_image_generator_set_applies_preferred_track_transform(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ applies: Bool
) {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    generator.appliesPreferredTrackTransform = applies
}

@_cdecl("av_asset_image_generator_set_maximum_size")
public func av_asset_image_generator_set_maximum_size(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ width: Double,
    _ height: Double
) {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    generator.maximumSize = CGSize(width: width, height: height)
}

@_cdecl("av_asset_image_generator_set_aperture_mode")
public func av_asset_image_generator_set_aperture_mode(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ modePtr: UnsafePointer<CChar>?
) {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    generator.apertureMode = modePtr.map { avpAssetImageGeneratorApertureMode(from: String(cString: $0)) }
}

@_cdecl("av_asset_image_generator_set_requested_time_tolerance_before")
public func av_asset_image_generator_set_requested_time_tolerance_before(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    generator.requestedTimeToleranceBefore = cmTime(value: value, timescale: timescale, kind: kind)
}

@_cdecl("av_asset_image_generator_set_requested_time_tolerance_after")
public func av_asset_image_generator_set_requested_time_tolerance_after(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    generator.requestedTimeToleranceAfter = cmTime(value: value, timescale: timescale, kind: kind)
}

@_cdecl("av_asset_image_generator_set_dynamic_range_policy")
public func av_asset_image_generator_set_dynamic_range_policy(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ policyPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetImageGenerator.dynamicRangePolicy requires macOS 15.0")
        return AVP_INVALID_ARGUMENT
    }
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    generator.dynamicRangePolicy = avpAssetImageGeneratorDynamicRangePolicy(from: String(cString: policyPtr))
    return AVP_OK
}

@_cdecl("av_asset_image_generator_copy_image_at_time")
public func av_asset_image_generator_copy_image_at_time(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ outActualValue: UnsafeMutablePointer<Int64>?,
    _ outActualTimescale: UnsafeMutablePointer<Int32>?,
    _ outActualKind: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    let requestedTime = cmTime(value: value, timescale: timescale, kind: kind)

    var actualTime = CMTime.invalid
    var image: CGImage?

    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetImageGenerator.image(at:) requires macOS 13.0")
        return nil
    }

    let status = avpBlockOnAsync(
        work: { try await generator.image(at: requestedTime) },
        onSuccess: { result in
            image = result.image
            actualTime = result.actualTime
        },
        outErrorMessage: outErrorMessage
    )
    guard status == AVP_OK, let image else { return nil }
    writeTime(actualTime, value: outActualValue, timescale: outActualTimescale, kind: outActualKind)
    return Unmanaged.passRetained(AVPAssetImageBox(image: image)).toOpaque()
}

@_cdecl("av_asset_image_generator_cancel_all_image_generation")
public func av_asset_image_generator_cancel_all_image_generation(
    _ generatorPtr: UnsafeMutableRawPointer
) {
    let generator = Unmanaged<AVAssetImageGenerator>.fromOpaque(generatorPtr).takeUnretainedValue()
    generator.cancelAllCGImageGeneration()
}

@_cdecl("av_asset_image_info_json")
public func av_asset_image_info_json(
    _ imagePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let image = Unmanaged<AVPAssetImageBox>.fromOpaque(imagePtr).takeUnretainedValue().image
    do {
        return ffiString(try avpEncodeJSON(makeAssetImagePayload(image)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

private func writeTime(
    _ time: CMTime,
    value: UnsafeMutablePointer<Int64>?,
    timescale: UnsafeMutablePointer<Int32>?,
    kind: UnsafeMutablePointer<Int32>?
) {
    let encoded = encodeTime(time)
    switch encoded.kind {
    case "numeric":
        value?.pointee = encoded.value ?? 0
        timescale?.pointee = encoded.timescale ?? 1
        kind?.pointee = 0
    case "indefinite":
        value?.pointee = 0
        timescale?.pointee = 0
        kind?.pointee = 2
    case "positive_infinity":
        value?.pointee = 0
        timescale?.pointee = 0
        kind?.pointee = 3
    case "negative_infinity":
        value?.pointee = 0
        timescale?.pointee = 0
        kind?.pointee = 4
    default:
        value?.pointee = 0
        timescale?.pointee = 0
        kind?.pointee = 1
    }
}
