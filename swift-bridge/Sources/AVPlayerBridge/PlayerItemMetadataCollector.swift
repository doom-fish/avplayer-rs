import AVPlayerABI
import AVFoundation
import Foundation

struct PlayerItemMediaDataCollectorInfoPayload: AVPBridge {
    let kind: String
}

struct MetadataCollectorInfoPayload: AVPBridge {
    let identifiers: [String]?
    let classifyingLabels: [String]?
    let hasDelegate: Bool
}

struct DateRangeMetadataGroupPayload: AVPBridge {
    let startDate: String
    let endDate: String?
    let classifyingLabel: String?
    let uniqueID: String?
    let items: [MetadataItemPayload]
}

struct MetadataCollectorEventPayload: AVPBridge {
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
    private let callback: AVPValueCallback
    private let gate: AVPCallbackGate

    init(
        collector: AVPlayerItemMetadataCollector,
        queue: DispatchQueue?,
        callback: @escaping AVPValueCallback,
        gate: AVPCallbackGate
    ) {
        self.collector = collector
        self.callback = callback
        self.gate = gate
        super.init()
        collector.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        collector?.setDelegate(nil, queue: nil)
        gate.finish()
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
        gate.deliverValue(payload, to: callback)
    }
}

@_cdecl("av_player_item_metadata_collector_create")
public func av_player_item_metadata_collector_create(
    _ identifiersValue: UnsafePointer<AVPValue>?,
    _ classifyingLabelsValue: UnsafePointer<AVPValue>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let identifiers = try identifiersValue.map { try avpDecodeValue($0, as: [String].self) }
        let classifyingLabels = try classifyingLabelsValue.map { try avpDecodeValue($0, as: [String].self) }
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

@_cdecl("av_player_item_metadata_collector_info_value")
public func av_player_item_metadata_collector_info_value(
    _ collectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let box = Unmanaged<AVPPlayerItemMetadataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue()
    let payload = MetadataCollectorInfoPayload(
        identifiers: box.identifiers,
        classifyingLabels: box.classifyingLabels,
        hasDelegate: box.metadataCollector.delegate != nil
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_media_data_collector_kind")
public func av_player_item_media_data_collector_kind(
    _ collectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let collector = Unmanaged<AVPPlayerItemMediaDataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue().collector
    if collector is AVPlayerItemMetadataCollector {
        return ffiString("metadata_collector")
    }
    return ffiString(String(describing: type(of: collector)))
}

@_cdecl("av_player_item_metadata_collector_add_observer")
public func av_player_item_metadata_collector_add_observer(
    _ collectorPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<AVPValue>?,
    _ callback: AVPValueCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing metadata collector observer callback")
        return nil
    }
    let collector = Unmanaged<AVPPlayerItemMetadataCollectorBox>.fromOpaque(collectorPtr).takeUnretainedValue().metadataCollector
    let observer = MetadataCollectorObserverBox(
        collector: collector,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_metadata_collector_observer_release")
public func av_player_item_metadata_collector_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    let observer = Unmanaged<MetadataCollectorObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}

@_cdecl("av_player_item_add_media_data_collector")
public func av_player_item_add_media_data_collector(
    _ itemPtr: UnsafeMutableRawPointer,
    _ collectorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
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

@_cdecl("av_player_item_media_data_collectors_value")
public func av_player_item_media_data_collectors_value(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let payload = item.mediaDataCollectors.map { collector in
        if collector is AVPlayerItemMetadataCollector {
            return PlayerItemMediaDataCollectorInfoPayload(kind: "metadata_collector")
        }
        return PlayerItemMediaDataCollectorInfoPayload(kind: String(describing: type(of: collector)))
    }
    do {
        return try avpEncodeValue(payload)
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

extension PlayerItemMediaDataCollectorInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            kind.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            kind: try binaryValue.field(0)
        )
    }
}

extension MetadataCollectorInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            identifiers.binaryValue,
            classifyingLabels.binaryValue,
            hasDelegate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            identifiers: try binaryValue.field(0),
            classifyingLabels: try binaryValue.field(1),
            hasDelegate: try binaryValue.field(2)
        )
    }
}

extension DateRangeMetadataGroupPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            startDate.binaryValue,
            endDate.binaryValue,
            classifyingLabel.binaryValue,
            uniqueID.binaryValue,
            items.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            startDate: try binaryValue.field(0),
            endDate: try binaryValue.field(1),
            classifyingLabel: try binaryValue.field(2),
            uniqueID: try binaryValue.field(3),
            items: try binaryValue.field(4)
        )
    }
}

extension MetadataCollectorEventPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            event.binaryValue,
            groups.binaryValue,
            newIndices.binaryValue,
            modifiedIndices.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            event: try binaryValue.field(0),
            groups: try binaryValue.field(1),
            newIndices: try binaryValue.field(2),
            modifiedIndices: try binaryValue.field(3)
        )
    }
}
