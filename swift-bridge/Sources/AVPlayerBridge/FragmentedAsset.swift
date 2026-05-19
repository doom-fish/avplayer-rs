import AVFoundation
import Foundation
import ObjectiveC.runtime

private struct AVPFragmentedAssetMinderPayload: Codable {
    let mindingInterval: Double
    let assetCount: Int
}

private struct AVPMediaExtensionPropertiesPayload: Codable {
    let containingBundleName: String?
    let containingBundleURL: String?
    let extensionIdentifier: String?
    let extensionName: String?
    let extensionURL: String?
}

private func avpBoolSelector(_ object: AnyObject, _ selectorName: String) -> Bool {
    let selector = NSSelectorFromString(selectorName)
    guard object.responds(to: selector) else { return false }
    typealias Fn = @convention(c) (AnyObject, Selector) -> ObjCBool
    let fn = unsafeBitCast(object.method(for: selector), to: Fn.self)
    return fn(object, selector).boolValue
}

private func avpClassBoolSelector(_ cls: AnyClass, _ selectorName: String) -> Bool {
    let selector = NSSelectorFromString(selectorName)
    guard let metaClass = object_getClass(cls), class_respondsToSelector(metaClass, selector) else {
        return false
    }
    typealias Fn = @convention(c) (AnyClass, Selector) -> ObjCBool
    let fn = unsafeBitCast(class_getMethodImplementation(metaClass, selector), to: Fn.self)
    return fn(cls, selector).boolValue
}

private func avpVoidSelector(_ object: AnyObject, _ selectorName: String, argument: AnyObject) {
    let selector = NSSelectorFromString(selectorName)
    guard object.responds(to: selector) else { return }
    typealias Fn = @convention(c) (AnyObject, Selector, AnyObject) -> Void
    let fn = unsafeBitCast(object.method(for: selector), to: Fn.self)
    fn(object, selector, argument)
}

@_cdecl("av_fragmented_asset_create")
public func av_fragmented_asset_create(
    _ urlPtr: UnsafePointer<CChar>,
    _ isFileURL: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let urlString = String(cString: urlPtr)
    if isFileURL {
        return avpRetained(AVFragmentedAsset(url: URL(fileURLWithPath: urlString), options: nil))
    }
    guard let url = URL(string: urlString) else {
        outErrorMessage?.pointee = ffiString("invalid fragmented asset URL")
        return nil
    }
    return avpRetained(AVFragmentedAsset(url: url, options: nil))
}

@_cdecl("av_fragmented_asset_track_count")
public func av_fragmented_asset_track_count(_ assetPtr: UnsafeMutableRawPointer) -> Int32 {
    let asset = Unmanaged<AVFragmentedAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return Int32(asset.tracks.count)
}

@_cdecl("av_fragmented_asset_copy_track_at_index")
public func av_fragmented_asset_copy_track_at_index(
    _ assetPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVFragmentedAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    let tracks = asset.tracks
    guard index >= 0, Int(index) < tracks.count else { return nil }
    return avpRetained(tracks[Int(index)])
}

@_cdecl("av_fragmented_asset_copy_track_with_id")
public func av_fragmented_asset_copy_track_with_id(
    _ assetPtr: UnsafeMutableRawPointer,
    _ trackID: Int32
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVFragmentedAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard let track = asset.track(withTrackID: CMPersistentTrackID(trackID)) else {
        return nil
    }
    return avpRetained(track)
}

@_cdecl("av_fragmented_asset_is_associated_with_minder")
public func av_fragmented_asset_is_associated_with_minder(_ assetPtr: UnsafeMutableRawPointer) -> Bool {
    let asset = Unmanaged<AVFragmentedAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return avpBoolSelector(asset, "isAssociatedWithFragmentMinder")
}

@_cdecl("av_fragmented_asset_expects_property_revised_notifications")
public func av_fragmented_asset_expects_property_revised_notifications() -> Bool {
    avpClassBoolSelector(AVFragmentedAsset.self, "expectsPropertyRevisedNotifications")
}

@_cdecl("av_fragmented_asset_is_playable_extended_mime_type")
public func av_fragmented_asset_is_playable_extended_mime_type(
    _ mimeTypePtr: UnsafePointer<CChar>
) -> Bool {
    AVFragmentedAsset.isPlayableExtendedMIMEType(String(cString: mimeTypePtr))
}

@_cdecl("av_fragmented_asset_track_segment_count")
public func av_fragmented_asset_track_segment_count(_ trackPtr: UnsafeMutableRawPointer) -> Int32 {
    let track = Unmanaged<AVFragmentedAssetTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    return Int32(track.segments.count)
}

@_cdecl("av_url_asset_copy_media_extension_properties")
public func av_url_asset_copy_media_extension_properties(
    _ assetPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else { return nil }
    let asset = Unmanaged<AVURLAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return asset.mediaExtensionProperties.map(avpRetained)
}

@_cdecl("av_media_extension_properties_info_json")
public func av_media_extension_properties_info_json(
    _ propertiesPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("media extension properties require macOS 15.0")
        return nil
    }
    let properties = Unmanaged<AVMediaExtensionProperties>.fromOpaque(propertiesPtr).takeUnretainedValue()
    let payload = AVPMediaExtensionPropertiesPayload(
        containingBundleName: properties.containingBundleName,
        containingBundleURL: properties.containingBundleURL.absoluteString,
        extensionIdentifier: properties.extensionIdentifier,
        extensionName: properties.extensionName,
        extensionURL: properties.extensionURL.absoluteString
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_fragmented_asset_minder_create")
public func av_fragmented_asset_minder_create(
    _ assetPtr: UnsafeMutableRawPointer,
    _ mindingInterval: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVFragmentedAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    return avpRetained(AVFragmentedAssetMinder(asset: asset, mindingInterval: mindingInterval))
}

@_cdecl("av_fragmented_asset_minder_info_json")
public func av_fragmented_asset_minder_info_json(
    _ minderPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let minder = Unmanaged<AVFragmentedAssetMinder>.fromOpaque(minderPtr).takeUnretainedValue()
    let payload = AVPFragmentedAssetMinderPayload(
        mindingInterval: minder.mindingInterval,
        assetCount: minder.assets.count
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_fragmented_asset_minder_copy_asset_at_index")
public func av_fragmented_asset_minder_copy_asset_at_index(
    _ minderPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let minder = Unmanaged<AVFragmentedAssetMinder>.fromOpaque(minderPtr).takeUnretainedValue()
    guard index >= 0, Int(index) < minder.assets.count else { return nil }
    guard let asset = minder.assets[Int(index)] as? AVFragmentedAsset else { return nil }
    return avpRetained(asset)
}

@_cdecl("av_fragmented_asset_minder_set_interval")
public func av_fragmented_asset_minder_set_interval(
    _ minderPtr: UnsafeMutableRawPointer,
    _ mindingInterval: Double
) {
    let minder = Unmanaged<AVFragmentedAssetMinder>.fromOpaque(minderPtr).takeUnretainedValue()
    minder.mindingInterval = mindingInterval
}

@_cdecl("av_fragmented_asset_minder_add_asset")
public func av_fragmented_asset_minder_add_asset(
    _ minderPtr: UnsafeMutableRawPointer,
    _ assetPtr: UnsafeMutableRawPointer
) {
    let minder = Unmanaged<AVFragmentedAssetMinder>.fromOpaque(minderPtr).takeUnretainedValue()
    let asset = Unmanaged<AVFragmentedAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    avpVoidSelector(minder, "addFragmentedAsset:", argument: asset)
}

@_cdecl("av_fragmented_asset_minder_remove_asset")
public func av_fragmented_asset_minder_remove_asset(
    _ minderPtr: UnsafeMutableRawPointer,
    _ assetPtr: UnsafeMutableRawPointer
) {
    let minder = Unmanaged<AVFragmentedAssetMinder>.fromOpaque(minderPtr).takeUnretainedValue()
    let asset = Unmanaged<AVFragmentedAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    avpVoidSelector(minder, "removeFragmentedAsset:", argument: asset)
}
