import AVFoundation
import Foundation

@_cdecl("av_player_looper_create")
public func av_player_looper_create(
    _ playerPtr: UnsafeMutableRawPointer,
    _ templateItemPtr: UnsafeMutableRawPointer,
    _ useLoopRange: Bool,
    _ startValue: Int64,
    _ startTimescale: Int32,
    _ startKind: Int32,
    _ durationValue: Int64,
    _ durationTimescale: Int32,
    _ durationKind: Int32,
    _ itemOrderingRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let templateItem = Unmanaged<AVPlayerItem>.fromOpaque(templateItemPtr).takeUnretainedValue()
    let loopRange = useLoopRange
        ? CMTimeRange(
            start: cmTime(value: startValue, timescale: startTimescale, kind: startKind),
            duration: cmTime(value: durationValue, timescale: durationTimescale, kind: durationKind)
        )
        : .invalid

    if #available(macOS 14.0, *) {
        let itemOrdering: AVPlayerLooper.ItemOrdering = itemOrderingRaw == 1
            ? .loopingItemsFollowExistingItems
            : .loopingItemsPrecedeExistingItems
        return Unmanaged.passRetained(
            AVPlayerLooper(player: player, templateItem: templateItem, timeRange: loopRange, existingItemsOrdering: itemOrdering)
        ).toOpaque()
    }

    if itemOrderingRaw != 0 {
        outErrorMessage?.pointee = ffiString("existingItemsOrdering requires macOS 14.0+")
        return nil
    }

    return Unmanaged.passRetained(AVPlayerLooper(player: player, templateItem: templateItem, timeRange: loopRange)).toOpaque()
}

@_cdecl("av_player_looper_release")
public func av_player_looper_release(_ looperPtr: UnsafeMutableRawPointer?) {
    guard let looperPtr else { return }
    Unmanaged<AVPlayerLooper>.fromOpaque(looperPtr).release()
}

@_cdecl("av_player_looper_info_json")
public func av_player_looper_info_json(
    _ looperPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let looper = Unmanaged<AVPlayerLooper>.fromOpaque(looperPtr).takeUnretainedValue()
    let payload = PlayerLooperInfoPayload(
        status: Int32(looper.status.rawValue),
        errorMessage: looper.error?.localizedDescription,
        loopCount: looper.loopCount,
        loopingItemCount: looper.loopingPlayerItems.count
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_looper_disable_looping")
public func av_player_looper_disable_looping(_ looperPtr: UnsafeMutableRawPointer) {
    let looper = Unmanaged<AVPlayerLooper>.fromOpaque(looperPtr).takeUnretainedValue()
    looper.disableLooping()
}

@_cdecl("av_player_looper_looping_item_count")
public func av_player_looper_looping_item_count(_ looperPtr: UnsafeMutableRawPointer) -> Int32 {
    let looper = Unmanaged<AVPlayerLooper>.fromOpaque(looperPtr).takeUnretainedValue()
    return Int32(looper.loopingPlayerItems.count)
}

@_cdecl("av_player_looper_copy_looping_item_at_index")
public func av_player_looper_copy_looping_item_at_index(
    _ looperPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let looper = Unmanaged<AVPlayerLooper>.fromOpaque(looperPtr).takeUnretainedValue()
    let items = looper.loopingPlayerItems
    guard index >= 0, Int(index) < items.count else { return nil }
    return Unmanaged.passRetained(items[Int(index)]).toOpaque()
}
