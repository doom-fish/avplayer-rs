import AVFoundation
import Foundation

@_cdecl("av_player_media_selection_criteria_create")
public func av_player_media_selection_criteria_create(
    _ preferredLanguagesJson: UnsafePointer<CChar>?,
    _ preferredMediaCharacteristicsJson: UnsafePointer<CChar>?,
    _ principalMediaCharacteristicsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let preferredLanguages = try preferredLanguagesJson.map { try avpDecodeJSON($0, as: [String].self) }
        let preferredMediaCharacteristics = try preferredMediaCharacteristicsJson.map {
            try avpDecodeJSON($0, as: [String].self).map(avpMediaCharacteristic(from:))
        }
        let principalMediaCharacteristics = try principalMediaCharacteristicsJson.map {
            try avpDecodeJSON($0, as: [String].self).map(avpMediaCharacteristic(from:))
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

@_cdecl("av_player_media_selection_criteria_info_json")
public func av_player_media_selection_criteria_info_json(
    _ criteriaPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let criteria = Unmanaged<AVPlayerMediaSelectionCriteria>.fromOpaque(criteriaPtr).takeUnretainedValue()
    let payload = MediaSelectionCriteriaPayload(
        preferredLanguages: criteria.preferredLanguages,
        preferredMediaCharacteristics: criteria.preferredMediaCharacteristics?.map(avpMediaCharacteristicString),
        principalMediaCharacteristics: criteria.principalMediaCharacteristics?.map(avpMediaCharacteristicString)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
