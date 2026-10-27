import AVPlayerABI
import AVFoundation
import Foundation

private struct AVPAssetExtraInfoPayload: AVPBridge {
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

private struct AVPAssetTrackExtraInfoPayload: AVPBridge {
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

@_cdecl("av_asset_extra_info_value")
public func av_asset_extra_info_value(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
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
        return try avpEncodeValue(payload)
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

@_cdecl("av_asset_track_extra_info_value")
public func av_asset_track_extra_info_value(
    _ trackPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
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
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

extension AVPAssetExtraInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            preferredRate.binaryValue,
            preferredVolume.binaryValue,
            overallDurationHint.binaryValue,
            availableMetadataFormats.binaryValue,
            availableChapterLocales.binaryValue,
            commonMetadata.binaryValue,
            creationDate.binaryValue,
            lyrics.binaryValue,
            hasProtectedContent.binaryValue,
            canContainFragments.binaryValue,
            containsFragments.binaryValue,
            playable.binaryValue,
            exportable.binaryValue,
            readable.binaryValue,
            composable.binaryValue,
            compatibleWithAirPlayVideo.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            preferredRate: try binaryValue.field(0),
            preferredVolume: try binaryValue.field(1),
            overallDurationHint: try binaryValue.field(2),
            availableMetadataFormats: try binaryValue.field(3),
            availableChapterLocales: try binaryValue.field(4),
            commonMetadata: try binaryValue.field(5),
            creationDate: try binaryValue.field(6),
            lyrics: try binaryValue.field(7),
            hasProtectedContent: try binaryValue.field(8),
            canContainFragments: try binaryValue.field(9),
            containsFragments: try binaryValue.field(10),
            playable: try binaryValue.field(11),
            exportable: try binaryValue.field(12),
            readable: try binaryValue.field(13),
            composable: try binaryValue.field(14),
            compatibleWithAirPlayVideo: try binaryValue.field(15)
        )
    }
}

extension AVPAssetTrackExtraInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            timeRange.binaryValue,
            languageCode.binaryValue,
            extendedLanguageTag.binaryValue,
            naturalTimeScale.binaryValue,
            preferredVolume.binaryValue,
            enabled.binaryValue,
            playable.binaryValue,
            decodable.binaryValue,
            selfContained.binaryValue,
            totalSampleDataLength.binaryValue,
            availableMetadataFormats.binaryValue,
            availableTrackAssociationTypes.binaryValue,
            canProvideSampleCursors.binaryValue,
            minFrameDuration.binaryValue,
            requiresFrameReordering.binaryValue,
            audible.binaryValue,
            visual.binaryValue,
            legible.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            timeRange: try binaryValue.field(0),
            languageCode: try binaryValue.field(1),
            extendedLanguageTag: try binaryValue.field(2),
            naturalTimeScale: try binaryValue.field(3),
            preferredVolume: try binaryValue.field(4),
            enabled: try binaryValue.field(5),
            playable: try binaryValue.field(6),
            decodable: try binaryValue.field(7),
            selfContained: try binaryValue.field(8),
            totalSampleDataLength: try binaryValue.field(9),
            availableMetadataFormats: try binaryValue.field(10),
            availableTrackAssociationTypes: try binaryValue.field(11),
            canProvideSampleCursors: try binaryValue.field(12),
            minFrameDuration: try binaryValue.field(13),
            requiresFrameReordering: try binaryValue.field(14),
            audible: try binaryValue.field(15),
            visual: try binaryValue.field(16),
            legible: try binaryValue.field(17)
        )
    }
}
