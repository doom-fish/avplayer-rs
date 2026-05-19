import AVFoundation
import Foundation

private struct AVPAssetExtraInfoPayload: Codable {
    let preferredRate: Float
    let preferredVolume: Float
    let overallDurationHint: TimePayload
    let availableMetadataFormats: [String]
    let availableChapterLocales: [String]
    let commonMetadata: [MetadataItemPayload]
    let creationDate: MetadataItemPayload?
    let lyrics: String?
    let hasProtectedContent: Bool
    let canContainFragments: Bool
    let containsFragments: Bool
    let playable: Bool
    let exportable: Bool
    let readable: Bool
    let composable: Bool
    let compatibleWithAirPlayVideo: Bool
}

private struct AVPAssetTrackExtraInfoPayload: Codable {
    let timeRange: TimeRangePayload
    let languageCode: String?
    let extendedLanguageTag: String?
    let naturalTimeScale: Int32
    let preferredVolume: Float
    let enabled: Bool
    let playable: Bool
    let decodable: Bool
    let selfContained: Bool
    let totalSampleDataLength: Int64
    let availableMetadataFormats: [String]
    let availableTrackAssociationTypes: [String]
    let canProvideSampleCursors: Bool
    let minFrameDuration: TimePayload
    let requiresFrameReordering: Bool
    let audible: Bool
    let visual: Bool
    let legible: Bool
}

@_cdecl("av_asset_extra_info_json")
public func av_asset_extra_info_json(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    let payload = AVPAssetExtraInfoPayload(
        preferredRate: asset.preferredRate,
        preferredVolume: asset.preferredVolume,
        overallDurationHint: encodeTime(asset.overallDurationHint),
        availableMetadataFormats: asset.availableMetadataFormats.map(\.rawValue),
        availableChapterLocales: asset.availableChapterLocales.map(\.identifier),
        commonMetadata: asset.commonMetadata.map(avpEncodeMetadataItem),
        creationDate: asset.creationDate.map(avpEncodeMetadataItem),
        lyrics: asset.lyrics,
        hasProtectedContent: asset.hasProtectedContent,
        canContainFragments: asset.canContainFragments,
        containsFragments: asset.containsFragments,
        playable: asset.isPlayable,
        exportable: asset.isExportable,
        readable: asset.isReadable,
        composable: asset.isComposable,
        compatibleWithAirPlayVideo: asset.isCompatibleWithAirPlayVideo
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_cancel_loading")
public func av_asset_cancel_loading(_ assetPtr: UnsafeMutableRawPointer) {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    asset.cancelLoading()
}

@_cdecl("av_asset_track_extra_info_json")
public func av_asset_track_extra_info_json(
    _ trackPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let track = Unmanaged<AVAssetTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    let payload = AVPAssetTrackExtraInfoPayload(
        timeRange: encodeTimeRange(track.timeRange),
        languageCode: track.languageCode,
        extendedLanguageTag: track.extendedLanguageTag,
        naturalTimeScale: Int32(track.naturalTimeScale),
        preferredVolume: track.preferredVolume,
        enabled: track.isEnabled,
        playable: track.isPlayable,
        decodable: track.isDecodable,
        selfContained: track.isSelfContained,
        totalSampleDataLength: track.totalSampleDataLength,
        availableMetadataFormats: track.availableMetadataFormats.map(\.rawValue),
        availableTrackAssociationTypes: track.availableTrackAssociationTypes.map(String.init(describing:)),
        canProvideSampleCursors: track.canProvideSampleCursors,
        minFrameDuration: encodeTime(track.minFrameDuration),
        requiresFrameReordering: track.requiresFrameReordering,
        audible: track.hasMediaCharacteristic(.audible),
        visual: track.hasMediaCharacteristic(.visual),
        legible: track.hasMediaCharacteristic(.legible)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
