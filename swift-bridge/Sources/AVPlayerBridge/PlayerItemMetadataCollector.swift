import AVFoundation
import Foundation

struct PlayerItemMediaDataCollectorInfoPayload: Codable {
    let kind: String
}

struct MetadataCollectorInfoPayload: Codable {
    let identifiers: [String]?
    let classifyingLabels: [String]?
    let hasDelegate: Bool
}

struct DateRangeMetadataGroupPayload: Codable {
    let startDate: String
    let endDate: String?
    let classifyingLabel: String?
    let uniqueID: String?
    let items: [MetadataItemPayload]
}

struct MetadataCollectorEventPayload: Codable {
    let event: String
    let groups: [DateRangeMetadataGroupPayload]
    let newIndices: [Int]
    let modifiedIndices: [Int]
}

class AVPPlayerItemMediaDataCollectorBox: NSObject {
    var collector: AVPlayerItemMediaDataCollector {
        fatalError("override in subclass")
    }
}

final class AVPPlayerItemMetadataCollectorBox: AVPPlayerItemMediaDataCollectorBox {
    let metadataCollector: AVPlayerItemMetadataCollector
    let identifiers: [String]?
    let classifyingLabels: [String]?

    init(
        metadataCollector: AVPlayerItemMetadataCollector,
        identifiers: [String]?,
        classifyingLabels: [String]?
    ) {
        self.metadataCollector = metadataCollector
        self.identifiers = identifiers
        self.classifyingLabels = classifyingLabels
    }

    override var collector: AVPlayerItemMediaDataCollector {
        metadataCollector
    }
}

final class MetadataCollectorObserverBox: NSObject, AVPlayerItemMetadataCollectorPushDelegate {
    private weak var collector: AVPlayerItemMetadataCollector?
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        collector: AVPlayerItemMetadataCollector,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.collector = collector
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        collector.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        collector?.setDelegate(nil, queue: nil)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    func metadataCollector(
        _ metadataCollector: AVPlayerItemMetadataCollector,
        didCollect metadataGroups: [AVDateRangeMetadataGroup],
        indexesOfNewGroups: IndexSet,
        indexesOfModifiedGroups: IndexSet
    ) {
        send(
            MetadataCollectorEventPayload(
                event: "did_collect_date_range_metadata_groups",
                groups: metadataGroups.map(encodeDateRangeMetadataGroup),
                newIndices: Array(indexesOfNewGroups),
                modifiedIndices: Array(indexesOfModifiedGroups)
            )
        )
    }

    private func send(_ payload: MetadataCollectorEventPayload) {
        guard !disposed else { return }
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
    }
}

@_cdecl("av_player_item_metadata_collector_create")
public func av_player_item_metadata_collector_create(
    _ identifiersJson: UnsafePointer<CChar>?,
    _ classifyingLabelsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let identifiers = try identifiersJson.map { try avpDecodeJSON($0, as: [String].self) }
        let classifyingLabels = try classifyingLabelsJson.map { try avpDecodeJSON($0, as: [String].self) }
        let collector = AVPlayerItemMetadataCollector(identifiers: identifiers, classifyingLabels: classifyingLabels)
        return Unmanaged.passRetained(
            AVPPlayerItemMetadataCollectorBox(
                metadataCollector: collector,
                identifiers: identifiers,
                classifyingLabels: classifyingLabels
            )
        ).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_metadata_collector_release")
public func av_player_item_metadata_collector_release(_ collectorPtr: UnsafeMutableRawPointer?) {
    guard let collectorPtr else { return }
    Unmanaged<AVPPlayerItemMediaDataCollectorBox>.fromOpaque(collectorPtr).release()
}

@_cdecl("av_player_item_metadata_collector_info_json")
public func av_player_item_metadata_collector_info_json(
    _ collectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let box = Unmanaged<AVPPlayerItemMetadataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue()
    let payload = MetadataCollectorInfoPayload(
        identifiers: box.identifiers,
        classifyingLabels: box.classifyingLabels,
        hasDelegate: box.metadataCollector.delegate != nil
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_media_data_collector_kind")
public func av_player_item_media_data_collector_kind(
    _ collectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let collector = Unmanaged<AVPPlayerItemMediaDataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue().collector
    if collector is AVPlayerItemMetadataCollector {
        return ffiString("metadata_collector")
    }
    return ffiString(String(describing: type(of: collector)))
}

@_cdecl("av_player_item_metadata_collector_add_observer")
public func av_player_item_metadata_collector_add_observer(
    _ collectorPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing metadata collector observer callback")
        return nil
    }
    let collector = Unmanaged<AVPPlayerItemMetadataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue().metadataCollector
    let observer = MetadataCollectorObserverBox(
        collector: collector,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_metadata_collector_observer_release")
public func av_player_item_metadata_collector_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<MetadataCollectorObserverBox>.fromOpaque(observerPtr).release()
}

@_cdecl("av_player_item_add_media_data_collector")
public func av_player_item_add_media_data_collector(
    _ itemPtr: UnsafeMutableRawPointer,
    _ collectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let collector = Unmanaged<AVPPlayerItemMediaDataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue().collector
    item.add(collector)
    return AVP_OK
}

@_cdecl("av_player_item_remove_media_data_collector")
public func av_player_item_remove_media_data_collector(
    _ itemPtr: UnsafeMutableRawPointer,
    _ collectorPtr: UnsafeMutableRawPointer
) {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let collector = Unmanaged<AVPPlayerItemMediaDataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue().collector
    item.remove(collector)
}

@_cdecl("av_player_item_media_data_collectors_json")
public func av_player_item_media_data_collectors_json(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let payload = item.mediaDataCollectors.map { collector in
        if collector is AVPlayerItemMetadataCollector {
            return PlayerItemMediaDataCollectorInfoPayload(kind: "metadata_collector")
        }
        return PlayerItemMediaDataCollectorInfoPayload(kind: String(describing: type(of: collector)))
    }
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

private func encodeDateRangeMetadataGroup(_ group: AVDateRangeMetadataGroup) -> DateRangeMetadataGroupPayload {
    DateRangeMetadataGroupPayload(
        startDate: avpISO8601String(group.startDate) ?? "",
        endDate: avpISO8601String(group.endDate),
        classifyingLabel: group.classifyingLabel,
        uniqueID: group.uniqueID,
        items: group.items.map(avpEncodeMetadataItem)
    )
}
