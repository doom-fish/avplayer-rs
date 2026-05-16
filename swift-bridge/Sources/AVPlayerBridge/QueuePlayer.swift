import AVFoundation
import Foundation

@_cdecl("av_queue_player_create")
public func av_queue_player_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    Unmanaged.passRetained(AVQueuePlayer()).toOpaque()
}

@_cdecl("av_queue_player_create_with_items")
public func av_queue_player_create_with_items(
    _ itemPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let items = try playerItemsFromPointers(itemPtrs, count: count)
        return Unmanaged.passRetained(AVQueuePlayer(items: items)).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_queue_player_release")
public func av_queue_player_release(_ playerPtr: UnsafeMutableRawPointer?) {
    guard let playerPtr else { return }
    Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).release()
}

@_cdecl("av_queue_player_item_count")
public func av_queue_player_item_count(_ playerPtr: UnsafeMutableRawPointer) -> Int32 {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    return Int32(player.items().count)
}

@_cdecl("av_queue_player_copy_item_at_index")
public func av_queue_player_copy_item_at_index(
    _ playerPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let items = player.items()
    guard index >= 0, Int(index) < items.count else { return nil }
    return Unmanaged.passRetained(items[Int(index)]).toOpaque()
}

@_cdecl("av_queue_player_advance_to_next_item")
public func av_queue_player_advance_to_next_item(_ playerPtr: UnsafeMutableRawPointer) {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.advanceToNextItem()
}

@_cdecl("av_queue_player_can_insert_item_after_item")
public func av_queue_player_can_insert_item_after_item(
    _ playerPtr: UnsafeMutableRawPointer,
    _ itemPtr: UnsafeMutableRawPointer,
    _ afterItemPtr: UnsafeMutableRawPointer?
) -> Bool {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let afterItem = afterItemPtr.map { Unmanaged<AVPlayerItem>.fromOpaque($0).takeUnretainedValue() }
    return player.canInsert(item, after: afterItem)
}

@_cdecl("av_queue_player_insert_item_after_item")
public func av_queue_player_insert_item_after_item(
    _ playerPtr: UnsafeMutableRawPointer,
    _ itemPtr: UnsafeMutableRawPointer,
    _ afterItemPtr: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let afterItem = afterItemPtr.map { Unmanaged<AVPlayerItem>.fromOpaque($0).takeUnretainedValue() }
    guard player.canInsert(item, after: afterItem) else {
        outErrorMessage?.pointee = ffiString("queue player cannot insert item at requested position")
        return AVP_OPERATION_FAILED
    }
    player.insert(item, after: afterItem)
    return AVP_OK
}

@_cdecl("av_queue_player_remove_item")
public func av_queue_player_remove_item(
    _ playerPtr: UnsafeMutableRawPointer,
    _ itemPtr: UnsafeMutableRawPointer
) {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    player.remove(item)
}

@_cdecl("av_queue_player_remove_all_items")
public func av_queue_player_remove_all_items(_ playerPtr: UnsafeMutableRawPointer) {
    let player = Unmanaged<AVQueuePlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    player.removeAllItems()
}

private func playerItemsFromPointers(
    _ itemPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    count: Int
) throws -> [AVPlayerItem] {
    guard let itemPtrs else {
        throw BridgeError.message("missing AVPlayerItem pointer array")
    }
    return (0..<count).map { index in
        Unmanaged<AVPlayerItem>.fromOpaque(itemPtrs[index]!).takeUnretainedValue()
    }
}
