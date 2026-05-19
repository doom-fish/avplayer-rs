import AVFoundation
import Foundation

private struct AVPMediaSelectionOptionPayload: Codable {
    let mediaType: String
    let mediaSubTypes: [UInt32]
    let playable: Bool
    let extendedLanguageTag: String?
    let localeIdentifier: String?
    let displayName: String
    let commonMetadata: [MetadataItemPayload]
    let availableMetadataFormats: [String]
}

@_cdecl("av_asset_available_media_selection_characteristics_json")
public func av_asset_available_media_selection_characteristics_json(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    do {
        return ffiString(
            try avpEncodeJSON(
                asset.availableMediaCharacteristicsWithMediaSelectionOptions.map(avpMediaCharacteristicString)
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_copy_media_selection_group_for_characteristic")
public func av_asset_copy_media_selection_group_for_characteristic(
    _ assetPtr: UnsafeMutableRawPointer,
    _ characteristicPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    let characteristic = avpMediaCharacteristic(from: String(cString: characteristicPtr))
    guard let group = asset.mediaSelectionGroup(forMediaCharacteristic: characteristic) else {
        return nil
    }
    return avpRetained(group)
}

@_cdecl("av_asset_copy_preferred_media_selection")
public func av_asset_copy_preferred_media_selection(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return avpRetained(asset.preferredMediaSelection)
}

@_cdecl("av_asset_media_selection_count")
public func av_asset_media_selection_count(_ assetPtr: UnsafeMutableRawPointer) -> Int32 {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return Int32(asset.allMediaSelections.count)
}

@_cdecl("av_asset_copy_media_selection_at_index")
public func av_asset_copy_media_selection_at_index(
    _ assetPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard index >= 0, Int(index) < asset.allMediaSelections.count else { return nil }
    return avpRetained(asset.allMediaSelections[Int(index)])
}

@_cdecl("av_media_selection_copy_selected_option")
public func av_media_selection_copy_selected_option(
    _ selectionPtr: UnsafeMutableRawPointer,
    _ groupPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let selection = Unmanaged<AVMediaSelection>.fromOpaque(selectionPtr).takeUnretainedValue()
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    guard let option = selection.selectedMediaOption(in: group) else { return nil }
    return avpRetained(option)
}

@_cdecl("av_media_selection_can_apply_automatically")
public func av_media_selection_can_apply_automatically(
    _ selectionPtr: UnsafeMutableRawPointer,
    _ groupPtr: UnsafeMutableRawPointer
) -> Bool {
    let selection = Unmanaged<AVMediaSelection>.fromOpaque(selectionPtr).takeUnretainedValue()
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    return selection.mediaSelectionCriteriaCanBeAppliedAutomatically(to: group)
}

@_cdecl("av_media_selection_mutable_copy")
public func av_media_selection_mutable_copy(
    _ selectionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let selection = Unmanaged<AVMediaSelection>.fromOpaque(selectionPtr).takeUnretainedValue()
    guard let copy = selection.mutableCopy() as? AVMutableMediaSelection else {
        outErrorMessage?.pointee = ffiString("failed to create mutable media selection copy")
        return nil
    }
    return avpRetained(copy)
}

@_cdecl("av_mutable_media_selection_select_option")
public func av_mutable_media_selection_select_option(
    _ selectionPtr: UnsafeMutableRawPointer,
    _ optionPtr: UnsafeMutableRawPointer?,
    _ groupPtr: UnsafeMutableRawPointer
) {
    let selection = Unmanaged<AVMutableMediaSelection>.fromOpaque(selectionPtr).takeUnretainedValue()
    let option = optionPtr.map { Unmanaged<AVMediaSelectionOption>.fromOpaque($0).takeUnretainedValue() }
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    selection.select(option, in: group)
}

@_cdecl("av_media_selection_group_option_count")
public func av_media_selection_group_option_count(_ groupPtr: UnsafeMutableRawPointer) -> Int32 {
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    return Int32(group.options.count)
}

@_cdecl("av_media_selection_group_copy_option_at_index")
public func av_media_selection_group_copy_option_at_index(
    _ groupPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    guard index >= 0, Int(index) < group.options.count else { return nil }
    return avpRetained(group.options[Int(index)])
}

@_cdecl("av_media_selection_group_copy_default_option")
public func av_media_selection_group_copy_default_option(
    _ groupPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    guard let option = group.defaultOption else { return nil }
    return avpRetained(option)
}

@_cdecl("av_media_selection_group_allows_empty_selection")
public func av_media_selection_group_allows_empty_selection(
    _ groupPtr: UnsafeMutableRawPointer
) -> Bool {
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    return group.allowsEmptySelection
}

@_cdecl("av_media_selection_group_copy_custom_media_selection_scheme")
public func av_media_selection_group_copy_custom_media_selection_scheme(
    _ groupPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("custom media selection schemes require macOS 26.0")
        return nil
    }
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    guard let scheme = group.customMediaSelectionScheme else { return nil }
    return avpRetained(scheme)
}

@_cdecl("av_media_selection_option_info_json")
public func av_media_selection_option_info_json(
    _ optionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let option = Unmanaged<AVMediaSelectionOption>.fromOpaque(optionPtr).takeUnretainedValue()
    let payload = AVPMediaSelectionOptionPayload(
        mediaType: avpMediaTypeString(option.mediaType),
        mediaSubTypes: option.mediaSubTypes.map { UInt32(truncating: $0) },
        playable: option.isPlayable,
        extendedLanguageTag: option.extendedLanguageTag,
        localeIdentifier: option.locale?.identifier,
        displayName: option.displayName,
        commonMetadata: option.commonMetadata.map(avpEncodeMetadataItem),
        availableMetadataFormats: option.availableMetadataFormats
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_media_selection_option_has_media_characteristic")
public func av_media_selection_option_has_media_characteristic(
    _ optionPtr: UnsafeMutableRawPointer,
    _ characteristicPtr: UnsafePointer<CChar>
) -> Bool {
    let option = Unmanaged<AVMediaSelectionOption>.fromOpaque(optionPtr).takeUnretainedValue()
    return option.hasMediaCharacteristic(avpMediaCharacteristic(from: String(cString: characteristicPtr)))
}

@_cdecl("av_media_selection_option_copy_associated_option")
public func av_media_selection_option_copy_associated_option(
    _ optionPtr: UnsafeMutableRawPointer,
    _ groupPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let option = Unmanaged<AVMediaSelectionOption>.fromOpaque(optionPtr).takeUnretainedValue()
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    guard let associated = option.associatedMediaSelectionOption(in: group) else { return nil }
    return avpRetained(associated)
}

@_cdecl("av_media_selection_option_display_name_for_locale_identifier")
public func av_media_selection_option_display_name_for_locale_identifier(
    _ optionPtr: UnsafeMutableRawPointer,
    _ localeIdentifierPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let option = Unmanaged<AVMediaSelectionOption>.fromOpaque(optionPtr).takeUnretainedValue()
    let locale = Locale(identifier: String(cString: localeIdentifierPtr))
    return ffiString(option.displayName(with: locale))
}

@_cdecl("av_custom_media_selection_scheme_should_offer_language_selection")
public func av_custom_media_selection_scheme_should_offer_language_selection(
    _ schemePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Bool {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("custom media selection schemes require macOS 26.0")
        return false
    }
    let scheme = Unmanaged<AVCustomMediaSelectionScheme>.fromOpaque(schemePtr).takeUnretainedValue()
    return scheme.shouldOfferLanguageSelection
}

@_cdecl("av_custom_media_selection_scheme_available_languages_json")
public func av_custom_media_selection_scheme_available_languages_json(
    _ schemePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("custom media selection schemes require macOS 26.0")
        return nil
    }
    let scheme = Unmanaged<AVCustomMediaSelectionScheme>.fromOpaque(schemePtr).takeUnretainedValue()
    do {
        return ffiString(try avpEncodeJSON(scheme.availableLanguages))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_custom_media_selection_scheme_selector_count")
public func av_custom_media_selection_scheme_selector_count(
    _ schemePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("custom media selection schemes require macOS 26.0")
        return 0
    }
    let scheme = Unmanaged<AVCustomMediaSelectionScheme>.fromOpaque(schemePtr).takeUnretainedValue()
    return Int32(scheme.selectors.count)
}

@_cdecl("av_custom_media_selection_scheme_copy_selector_at_index")
public func av_custom_media_selection_scheme_copy_selector_at_index(
    _ schemePtr: UnsafeMutableRawPointer,
    _ index: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("custom media selection schemes require macOS 26.0")
        return nil
    }
    let scheme = Unmanaged<AVCustomMediaSelectionScheme>.fromOpaque(schemePtr).takeUnretainedValue()
    guard index >= 0, Int(index) < scheme.selectors.count else { return nil }
    return avpRetained(scheme.selectors[Int(index)])
}

@_cdecl("av_media_presentation_selector_identifier")
public func av_media_presentation_selector_identifier(
    _ selectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("media presentation selectors require macOS 26.0")
        return nil
    }
    let selector = Unmanaged<AVMediaPresentationSelector>.fromOpaque(selectorPtr).takeUnretainedValue()
    return ffiString(selector.identifier)
}

@_cdecl("av_media_presentation_selector_display_name_for_locale_identifier")
public func av_media_presentation_selector_display_name_for_locale_identifier(
    _ selectorPtr: UnsafeMutableRawPointer,
    _ localeIdentifierPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("media presentation selectors require macOS 26.0")
        return nil
    }
    let selector = Unmanaged<AVMediaPresentationSelector>.fromOpaque(selectorPtr).takeUnretainedValue()
    return ffiString(selector.displayName(forLocaleIdentifier: String(cString: localeIdentifierPtr)))
}

@_cdecl("av_media_presentation_selector_setting_count")
public func av_media_presentation_selector_setting_count(
    _ selectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("media presentation selectors require macOS 26.0")
        return 0
    }
    let selector = Unmanaged<AVMediaPresentationSelector>.fromOpaque(selectorPtr).takeUnretainedValue()
    return Int32(selector.settings.count)
}

@_cdecl("av_media_presentation_selector_copy_setting_at_index")
public func av_media_presentation_selector_copy_setting_at_index(
    _ selectorPtr: UnsafeMutableRawPointer,
    _ index: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("media presentation selectors require macOS 26.0")
        return nil
    }
    let selector = Unmanaged<AVMediaPresentationSelector>.fromOpaque(selectorPtr).takeUnretainedValue()
    guard index >= 0, Int(index) < selector.settings.count else { return nil }
    return avpRetained(selector.settings[Int(index)])
}

@_cdecl("av_media_presentation_setting_media_characteristic")
public func av_media_presentation_setting_media_characteristic(
    _ settingPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("media presentation settings require macOS 26.0")
        return nil
    }
    let setting = Unmanaged<AVMediaPresentationSetting>.fromOpaque(settingPtr).takeUnretainedValue()
    return ffiString(avpMediaCharacteristicString(setting.mediaCharacteristic))
}

@_cdecl("av_media_presentation_setting_display_name_for_locale_identifier")
public func av_media_presentation_setting_display_name_for_locale_identifier(
    _ settingPtr: UnsafeMutableRawPointer,
    _ localeIdentifierPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("media presentation settings require macOS 26.0")
        return nil
    }
    let setting = Unmanaged<AVMediaPresentationSetting>.fromOpaque(settingPtr).takeUnretainedValue()
    return ffiString(setting.displayName(forLocaleIdentifier: String(cString: localeIdentifierPtr)))
}
