import AVFoundation
import Foundation

private struct AssetCacheInfoPayload: Codable {
    let playableOffline: Bool
}

@_cdecl("av_url_asset_copy_asset_cache")
public func av_url_asset_copy_asset_cache(
    _ assetPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard let urlAsset = asset as? AVURLAsset, let cache = urlAsset.assetCache else {
        return nil
    }
    return Unmanaged.passRetained(cache).toOpaque()
}

@_cdecl("av_asset_cache_info_json")
public func av_asset_cache_info_json(
    _ cachePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let cache = Unmanaged<AVAssetCache>.fromOpaque(cachePtr).takeUnretainedValue()
    do {
        return ffiString(try avpEncodeJSON(AssetCacheInfoPayload(playableOffline: cache.isPlayableOffline)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_cache_media_selection_option_count")
public func av_asset_cache_media_selection_option_count(
    _ cachePtr: UnsafeMutableRawPointer,
    _ groupPtr: UnsafeMutableRawPointer
) -> Int32 {
    let cache = Unmanaged<AVAssetCache>.fromOpaque(cachePtr).takeUnretainedValue()
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    return Int32(cache.mediaSelectionOptions(in: group).count)
}

@_cdecl("av_asset_cache_copy_media_selection_option_at_index")
public func av_asset_cache_copy_media_selection_option_at_index(
    _ cachePtr: UnsafeMutableRawPointer,
    _ groupPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let cache = Unmanaged<AVAssetCache>.fromOpaque(cachePtr).takeUnretainedValue()
    let group = Unmanaged<AVMediaSelectionGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    let options = cache.mediaSelectionOptions(in: group)
    guard index >= 0, Int(index) < options.count else { return nil }
    return Unmanaged.passRetained(options[Int(index)]).toOpaque()
}
