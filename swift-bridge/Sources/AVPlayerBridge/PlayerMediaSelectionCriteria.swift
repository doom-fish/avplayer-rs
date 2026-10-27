import AVPlayerABI
import AVFoundation
import Foundation

@_cdecl("av_player_media_selection_criteria_create")
public func av_player_media_selection_criteria_create(
    _ preferredLanguagesValue: UnsafePointer<AVPValue>?,
    _ preferredMediaCharacteristicsValue: UnsafePointer<AVPValue>?,
    _ principalMediaCharacteristicsValue: UnsafePointer<AVPValue>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let preferredLanguages = try preferredLanguagesValue.map { try avpDecodeValue($0, as: [String].self) }
        let preferredMediaCharacteristics = try preferredMediaCharacteristicsValue.map {
            try avpDecodeValue($0, as: [String].self).map(avpMediaCharacteristic(from:))
        }
        let principalMediaCharacteristics = try principalMediaCharacteristicsValue.map {
            try avpDecodeValue($0, as: [String].self).map(avpMediaCharacteristic(from:))
        }
        let criteria = if let principalMediaCharacteristics {
            AVPlayerMediaSelectionCriteria(
                principalMediaCharacteristics: principalMediaCharacteristics,
                preferredLanguages: preferredLanguages,
                preferredMediaCharacteristics: preferredMediaCharacteristics
            )
        } else {
            AVPlayerMediaSelectionCriteria(
                preferredLanguages: preferredLanguages,
                preferredMediaCharacteristics: preferredMediaCharacteristics
            )
        }
        return Unmanaged.passRetained(criteria).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_media_selection_criteria_release")
public func av_player_media_selection_criteria_release(_ criteriaPtr: UnsafeMutableRawPointer?) {
    guard let criteriaPtr else { return }
    Unmanaged<AVPlayerMediaSelectionCriteria>.fromOpaque(criteriaPtr).release()
}

@_cdecl("av_player_media_selection_criteria_info_value")
public func av_player_media_selection_criteria_info_value(
    _ criteriaPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let criteria = Unmanaged<AVPlayerMediaSelectionCriteria>.fromOpaque(criteriaPtr).takeUnretainedValue()
    let payload = MediaSelectionCriteriaPayload(
        preferredLanguages: criteria.preferredLanguages,
        preferredMediaCharacteristics: criteria.preferredMediaCharacteristics?.map(avpMediaCharacteristicString),
        principalMediaCharacteristics: criteria.principalMediaCharacteristics?.map(avpMediaCharacteristicString)
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
