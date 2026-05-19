import AVFoundation
import Foundation

private struct AVPAssetVariantPayload: Codable {
    let peakBitRate: Double?
    let averageBitRate: Double?
    let url: String?
}

private struct AVPAssetVariantVideoAttributesPayload: Codable {
    let videoRange: String
    let codecTypes: [UInt32]
    let presentationSize: SizePayload
    let nominalFrameRate: Double?
}

private struct AVPAssetVariantVideoLayoutAttributesPayload: Codable {
    let stereoViewComponents: UInt32
    let projectionType: String
}

private struct AVPAssetVariantAudioAttributesPayload: Codable {
    let formatIds: [UInt32]
}

private struct AVPAssetVariantAudioRenditionPayload: Codable {
    let channelCount: Int?
    let binaural: Bool?
    let immersive: Bool?
    let downmix: Bool?
}

@_cdecl("av_asset_variant_count")
public func av_asset_variant_count(_ assetPtr: UnsafeMutableRawPointer) -> Int32 {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard #available(macOS 12.0, *), let urlAsset = asset as? AVURLAsset else { return 0 }
    return Int32(urlAsset.variants.count)
}

@_cdecl("av_asset_copy_variant_at_index")
public func av_asset_copy_variant_at_index(
    _ assetPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard #available(macOS 12.0, *), let urlAsset = asset as? AVURLAsset else { return nil }
    guard index >= 0, Int(index) < urlAsset.variants.count else { return nil }
    return avpRetained(urlAsset.variants[Int(index)])
}

@_cdecl("av_asset_variant_info_json")
public func av_asset_variant_info_json(
    _ variantPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("asset variants require macOS 12.0")
        return nil
    }
    let variant = Unmanaged<AVAssetVariant>.fromOpaque(variantPtr).takeUnretainedValue()
    let payload = AVPAssetVariantPayload(
        peakBitRate: variant.peakBitRate,
        averageBitRate: variant.averageBitRate,
        url: {
            if #available(macOS 26.0, *) {
                return variant.url.absoluteString
            }
            return nil
        }()
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_variant_copy_video_attributes")
public func av_asset_variant_copy_video_attributes(
    _ variantPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    let variant = Unmanaged<AVAssetVariant>.fromOpaque(variantPtr).takeUnretainedValue()
    guard let value = variant.videoAttributes else { return nil }
    return avpRetained(value)
}

@_cdecl("av_asset_variant_copy_audio_attributes")
public func av_asset_variant_copy_audio_attributes(
    _ variantPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    let variant = Unmanaged<AVAssetVariant>.fromOpaque(variantPtr).takeUnretainedValue()
    guard let value = variant.audioAttributes else { return nil }
    return avpRetained(value)
}

@_cdecl("av_asset_variant_video_attributes_info_json")
public func av_asset_variant_video_attributes_info_json(
    _ attributesPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("variant video attributes require macOS 12.0")
        return nil
    }
    let attributes = Unmanaged<AVAssetVariant.VideoAttributes>.fromOpaque(attributesPtr).takeUnretainedValue()
    let payload = AVPAssetVariantVideoAttributesPayload(
        videoRange: attributes.videoRange.rawValue,
        codecTypes: attributes.codecTypes.map { $0 },
        presentationSize: encodeSize(attributes.presentationSize),
        nominalFrameRate: attributes.nominalFrameRate
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_variant_video_layout_attribute_count")
public func av_asset_variant_video_layout_attribute_count(
    _ attributesPtr: UnsafeMutableRawPointer
) -> Int32 {
    guard #available(macOS 14.0, *) else { return 0 }
    let attributes = Unmanaged<AVAssetVariant.VideoAttributes>.fromOpaque(attributesPtr).takeUnretainedValue()
    return Int32(attributes.videoLayoutAttributes.count)
}

@_cdecl("av_asset_variant_video_layout_attribute_copy_at_index")
public func av_asset_variant_video_layout_attribute_copy_at_index(
    _ attributesPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.0, *) else { return nil }
    let attributes = Unmanaged<AVAssetVariant.VideoAttributes>.fromOpaque(attributesPtr).takeUnretainedValue()
    guard index >= 0, Int(index) < attributes.videoLayoutAttributes.count else { return nil }
    return avpRetained(attributes.videoLayoutAttributes[Int(index)])
}

@_cdecl("av_asset_variant_video_layout_attributes_info_json")
public func av_asset_variant_video_layout_attributes_info_json(
    _ attributesPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("variant video layout attributes require macOS 14.0")
        return nil
    }
    let attributes = Unmanaged<AVAssetVariant.VideoAttributes.LayoutAttributes>.fromOpaque(attributesPtr).takeUnretainedValue()
    let payload = AVPAssetVariantVideoLayoutAttributesPayload(
        stereoViewComponents: UInt32(truncatingIfNeeded: attributes.stereoViewComponents.rawValue),
        projectionType: String(describing: attributes.projectionType)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_variant_audio_attributes_info_json")
public func av_asset_variant_audio_attributes_info_json(
    _ attributesPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("variant audio attributes require macOS 12.0")
        return nil
    }
    let attributes = Unmanaged<AVAssetVariant.AudioAttributes>.fromOpaque(attributesPtr).takeUnretainedValue()
    let payload = AVPAssetVariantAudioAttributesPayload(
        formatIds: attributes.formatIDs.map { $0 }
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_variant_audio_attributes_copy_rendition_specific_attributes")
public func av_asset_variant_audio_attributes_copy_rendition_specific_attributes(
    _ attributesPtr: UnsafeMutableRawPointer,
    _ optionPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    let attributes = Unmanaged<AVAssetVariant.AudioAttributes>.fromOpaque(attributesPtr).takeUnretainedValue()
    let option = Unmanaged<AVMediaSelectionOption>.fromOpaque(optionPtr).takeUnretainedValue()
    guard let value = attributes.renditionSpecificAttributes(for: option) else {
        return nil
    }
    return avpRetained(value)
}

@_cdecl("av_asset_variant_audio_rendition_info_json")
public func av_asset_variant_audio_rendition_info_json(
    _ attributesPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("variant audio rendition attributes require macOS 12.0")
        return nil
    }
    let attributes = Unmanaged<AVAssetVariant.AudioAttributes.RenditionSpecificAttributes>.fromOpaque(attributesPtr).takeUnretainedValue()
    let payload = AVPAssetVariantAudioRenditionPayload(
        channelCount: attributes.channelCount,
        binaural: {
            if #available(macOS 13.0, *) {
                return attributes.isBinaural
            }
            return nil
        }(),
        immersive: {
            if #available(macOS 14.0, *) {
                return attributes.isImmersive
            }
            return nil
        }(),
        downmix: {
            if #available(macOS 13.0, *) {
                return attributes.isDownmix
            }
            return nil
        }()
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_variant_qualifier_create_with_variant")
public func av_asset_variant_qualifier_create_with_variant(
    _ variantPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("asset variant qualifiers require macOS 12.0")
        return nil
    }
    let variant = Unmanaged<AVAssetVariant>.fromOpaque(variantPtr).takeUnretainedValue()
    return avpRetained(AVAssetVariantQualifier(variant: variant))
}
