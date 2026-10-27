import AVPlayerABI
import AVFoundation
import CoreMedia
import Foundation

private struct AVPMetadataGroupPayload: AVPBridge {
    let items: [MetadataItemPayload]
    let classifyingLabel: String?
    let uniqueId: String?
}

private struct AVPTimedMetadataGroupPayload: AVPBridge {
    let items: [MetadataItemPayload]
    let classifyingLabel: String?
    let uniqueId: String?
    let timeRange: TimeRangePayload
}

private struct AVPDateRangeMetadataGroupPayload: AVPBridge {
    let items: [MetadataItemPayload]
    let classifyingLabel: String?
    let uniqueId: String?
    let startDate: String
    let endDate: String?
}

private struct AVPMutableMetadataItemPayload: AVPBridge {
    let identifier: String?
    let extendedLanguageTag: String?
    let localeIdentifier: String?
    let time: TimePayload
    let duration: TimePayload
    let dataType: String?
    let stringValue: String?
    let numberValue: Double?
    let valueDescription: String?
    let startDate: String?
    let keySpace: String?
    let keyString: String?
}

private func encodeMetadataGroup(_ group: AVMetadataGroup) -> AVPMetadataGroupPayload {
    AVPMetadataGroupPayload(
        items: group.items.map(avpEncodeMetadataItem),
        classifyingLabel: group.classifyingLabel,
        uniqueId: group.uniqueID
    )
}

private func encodeTimedMetadataGroup(_ group: AVTimedMetadataGroup) -> AVPTimedMetadataGroupPayload {
    AVPTimedMetadataGroupPayload(
        items: group.items.map(avpEncodeMetadataItem),
        classifyingLabel: group.classifyingLabel,
        uniqueId: group.uniqueID,
        timeRange: encodeTimeRange(group.timeRange)
    )
}

private func encodeDateRangeMetadataGroupHandle(
    _ group: AVDateRangeMetadataGroup
) -> AVPDateRangeMetadataGroupPayload {
    AVPDateRangeMetadataGroupPayload(
        items: group.items.map(avpEncodeMetadataItem),
        classifyingLabel: group.classifyingLabel,
        uniqueId: group.uniqueID,
        startDate: avpISO8601String(group.startDate) ?? "",
        endDate: avpISO8601String(group.endDate)
    )
}

private func decodeMetadataItems(
    _ itemsValue: UnsafePointer<AVPValue>?
) throws -> [AVMetadataItem] {
    let payloads = try avpDecodeValue(itemsValue, as: [MetadataItemPayload].self)
    return avpMetadataItems(from: payloads)
}

private func decodeDate(
    _ value: UnsafePointer<AVPValue>?,
    parameterName: String
) throws -> Date {
    let raw = String(bridge: value!)
    guard let date = avpDate(fromISO8601: raw) else {
        throw BridgeError.message("invalid ISO-8601 date for \(parameterName)")
    }
    return date
}

@_cdecl("av_metadata_group_info_value")
public func av_metadata_group_info_value(
    _ groupPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let group = Unmanaged<AVMetadataGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    do {
        return try avpEncodeValue(encodeMetadataGroup(group))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_timed_metadata_group_create")
public func av_timed_metadata_group_create(
    _ itemsValue: UnsafePointer<AVPValue>,
    _ startValue: Int64,
    _ startTimescale: Int32,
    _ startKind: Int32,
    _ durationValue: Int64,
    _ durationTimescale: Int32,
    _ durationKind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let group = AVTimedMetadataGroup(
            items: try decodeMetadataItems(itemsValue),
            timeRange: CMTimeRange(
                start: cmTime(value: startValue, timescale: startTimescale, kind: startKind),
                duration: cmTime(value: durationValue, timescale: durationTimescale, kind: durationKind)
            )
        )
        return avpRetained(group)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_timed_metadata_group_info_value")
public func av_timed_metadata_group_info_value(
    _ groupPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let group = Unmanaged<AVTimedMetadataGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    do {
        return try avpEncodeValue(encodeTimedMetadataGroup(group))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_mutable_timed_metadata_group_create")
public func av_mutable_timed_metadata_group_create(
    _ itemsValue: UnsafePointer<AVPValue>,
    _ startValue: Int64,
    _ startTimescale: Int32,
    _ startKind: Int32,
    _ durationValue: Int64,
    _ durationTimescale: Int32,
    _ durationKind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let group = AVMutableTimedMetadataGroup(
            items: try decodeMetadataItems(itemsValue),
            timeRange: CMTimeRange(
                start: cmTime(value: startValue, timescale: startTimescale, kind: startKind),
                duration: cmTime(value: durationValue, timescale: durationTimescale, kind: durationKind)
            )
        )
        return avpRetained(group)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_mutable_timed_metadata_group_set_time_range")
public func av_mutable_timed_metadata_group_set_time_range(
    _ groupPtr: UnsafeMutableRawPointer,
    _ startValue: Int64,
    _ startTimescale: Int32,
    _ startKind: Int32,
    _ durationValue: Int64,
    _ durationTimescale: Int32,
    _ durationKind: Int32
) {
    let group = Unmanaged<AVMutableTimedMetadataGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    group.timeRange = CMTimeRange(
        start: cmTime(value: startValue, timescale: startTimescale, kind: startKind),
        duration: cmTime(value: durationValue, timescale: durationTimescale, kind: durationKind)
    )
}

@_cdecl("av_mutable_timed_metadata_group_set_items_value")
public func av_mutable_timed_metadata_group_set_items_value(
    _ groupPtr: UnsafeMutableRawPointer,
    _ itemsValue: UnsafePointer<AVPValue>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    let group = Unmanaged<AVMutableTimedMetadataGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    do {
        group.items = try decodeMetadataItems(itemsValue)
        return AVP_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVP_OPERATION_FAILED
    }
}

@_cdecl("av_date_range_metadata_group_create")
public func av_date_range_metadata_group_create(
    _ itemsValue: UnsafePointer<AVPValue>,
    _ startDatePtr: UnsafePointer<AVPValue>,
    _ endDatePtr: UnsafePointer<AVPValue>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let group = AVDateRangeMetadataGroup(
            items: try decodeMetadataItems(itemsValue),
            start: try decodeDate(startDatePtr, parameterName: "startDate"),
            end: avpDate(fromISO8601: endDatePtr.map(String.init(bridge:)))
        )
        return avpRetained(group)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_date_range_metadata_group_info_value")
public func av_date_range_metadata_group_info_value(
    _ groupPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let group = Unmanaged<AVDateRangeMetadataGroup>.fromOpaque(groupPtr).takeUnretainedValue()
    do {
        return try avpEncodeValue(encodeDateRangeMetadataGroupHandle(group))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_mutable_date_range_metadata_group_create")
public func av_mutable_date_range_metadata_group_create(
    _ itemsValue: UnsafePointer<AVPValue>,
    _ startDatePtr: UnsafePointer<AVPValue>,
    _ endDatePtr: UnsafePointer<AVPValue>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let group = AVMutableDateRangeMetadataGroup(
            items: try decodeMetadataItems(itemsValue),
            start: try decodeDate(startDatePtr, parameterName: "startDate"),
            end: avpDate(fromISO8601: endDatePtr.map(String.init(bridge:)))
        )
        return avpRetained(group)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_mutable_metadata_item_create")
public func av_mutable_metadata_item_create() -> UnsafeMutableRawPointer {
    avpRetained(AVMutableMetadataItem())
}

@_cdecl("av_mutable_metadata_item_info_value")
public func av_mutable_metadata_item_info_value(
    _ itemPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let payload = AVPMutableMetadataItemPayload(
        identifier: item.identifier?.rawValue,
        extendedLanguageTag: item.extendedLanguageTag,
        localeIdentifier: item.locale?.identifier,
        time: encodeTime(item.time),
        duration: encodeTime(item.duration),
        dataType: item.dataType,
        stringValue: item.stringValue,
        numberValue: item.numberValue?.doubleValue,
        valueDescription: item.value.map(String.init(describing:)),
        startDate: avpISO8601String(item.startDate),
        keySpace: item.keySpace?.rawValue,
        keyString: item.key.map(String.init(describing:))
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_mutable_metadata_item_set_identifier")
public func av_mutable_metadata_item_set_identifier(
    _ itemPtr: UnsafeMutableRawPointer,
    _ identifierPtr: UnsafePointer<AVPValue>?
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.identifier = identifierPtr.map { AVMetadataIdentifier(rawValue: String(bridge: $0)) }
}

@_cdecl("av_mutable_metadata_item_set_extended_language_tag")
public func av_mutable_metadata_item_set_extended_language_tag(
    _ itemPtr: UnsafeMutableRawPointer,
    _ languageTagPtr: UnsafePointer<AVPValue>?
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.extendedLanguageTag = languageTagPtr.map { String(bridge: $0) }
}

@_cdecl("av_mutable_metadata_item_set_locale_identifier")
public func av_mutable_metadata_item_set_locale_identifier(
    _ itemPtr: UnsafeMutableRawPointer,
    _ localeIdentifierPtr: UnsafePointer<AVPValue>?
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.locale = localeIdentifierPtr.map { Locale(identifier: String(bridge: $0)) }
}

@_cdecl("av_mutable_metadata_item_set_time")
public func av_mutable_metadata_item_set_time(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.time = cmTime(value: value, timescale: timescale, kind: kind)
}

@_cdecl("av_mutable_metadata_item_set_duration")
public func av_mutable_metadata_item_set_duration(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.duration = cmTime(value: value, timescale: timescale, kind: kind)
}

@_cdecl("av_mutable_metadata_item_set_data_type")
public func av_mutable_metadata_item_set_data_type(
    _ itemPtr: UnsafeMutableRawPointer,
    _ dataTypePtr: UnsafePointer<AVPValue>?
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.dataType = dataTypePtr.map { String(bridge: $0) }
}

@_cdecl("av_mutable_metadata_item_set_string_value")
public func av_mutable_metadata_item_set_string_value(
    _ itemPtr: UnsafeMutableRawPointer,
    _ valuePtr: UnsafePointer<AVPValue>?
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.value = valuePtr.map { String(bridge: $0) as NSString }
}

@_cdecl("av_mutable_metadata_item_set_number_value")
public func av_mutable_metadata_item_set_number_value(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Double
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.value = NSNumber(value: value)
}

@_cdecl("av_mutable_metadata_item_clear_value")
public func av_mutable_metadata_item_clear_value(_ itemPtr: UnsafeMutableRawPointer) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.value = nil
}

@_cdecl("av_mutable_metadata_item_set_start_date")
public func av_mutable_metadata_item_set_start_date(
    _ itemPtr: UnsafeMutableRawPointer,
    _ startDatePtr: UnsafePointer<AVPValue>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    if let startDatePtr {
        guard let date = avpDate(fromISO8601: String(bridge: startDatePtr)) else {
            outErrorMessage?.pointee = ffiString("invalid ISO-8601 date for startDate")
            return AVP_OPERATION_FAILED
        }
        item.startDate = date
    } else {
        item.startDate = nil
    }
    return AVP_OK
}

@_cdecl("av_mutable_metadata_item_set_key_space")
public func av_mutable_metadata_item_set_key_space(
    _ itemPtr: UnsafeMutableRawPointer,
    _ keySpacePtr: UnsafePointer<AVPValue>?
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.keySpace = keySpacePtr.map { AVMetadataKeySpace(rawValue: String(bridge: $0)) }
}

@_cdecl("av_mutable_metadata_item_set_key_string")
public func av_mutable_metadata_item_set_key_string(
    _ itemPtr: UnsafeMutableRawPointer,
    _ keyPtr: UnsafePointer<AVPValue>?
) {
    let item = Unmanaged<AVMutableMetadataItem>.fromOpaque(itemPtr).takeUnretainedValue()
    item.key = keyPtr.map { String(bridge: $0) as NSString }
}

@_cdecl("av_metadata_item_filter_create_for_sharing")
public func av_metadata_item_filter_create_for_sharing() -> UnsafeMutableRawPointer {
    avpRetained(AVMetadataItemFilter.forSharing())
}

@_cdecl("av_metadata_item_filter_filter_value")
public func av_metadata_item_filter_filter_value(
    _ filterPtr: UnsafeMutableRawPointer,
    _ itemsValue: UnsafePointer<AVPValue>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let filter = Unmanaged<AVMetadataItemFilter>.fromOpaque(filterPtr).takeUnretainedValue()
    do {
        let items = try decodeMetadataItems(itemsValue)
        let filtered = AVMetadataItem.metadataItems(from: items, filteredBy: filter)
        return try avpEncodeValue(filtered.map(avpEncodeMetadataItem))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

extension AVPMetadataGroupPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            items.binaryValue,
            classifyingLabel.binaryValue,
            uniqueId.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            items: try binaryValue.field(0),
            classifyingLabel: try binaryValue.field(1),
            uniqueId: try binaryValue.field(2)
        )
    }
}

extension AVPTimedMetadataGroupPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            items.binaryValue,
            classifyingLabel.binaryValue,
            uniqueId.binaryValue,
            timeRange.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            items: try binaryValue.field(0),
            classifyingLabel: try binaryValue.field(1),
            uniqueId: try binaryValue.field(2),
            timeRange: try binaryValue.field(3)
        )
    }
}

extension AVPDateRangeMetadataGroupPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            items.binaryValue,
            classifyingLabel.binaryValue,
            uniqueId.binaryValue,
            startDate.binaryValue,
            endDate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            items: try binaryValue.field(0),
            classifyingLabel: try binaryValue.field(1),
            uniqueId: try binaryValue.field(2),
            startDate: try binaryValue.field(3),
            endDate: try binaryValue.field(4)
        )
    }
}

extension AVPMutableMetadataItemPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            identifier.binaryValue,
            extendedLanguageTag.binaryValue,
            localeIdentifier.binaryValue,
            time.binaryValue,
            duration.binaryValue,
            dataType.binaryValue,
            stringValue.binaryValue,
            numberValue.binaryValue,
            valueDescription.binaryValue,
            startDate.binaryValue,
            keySpace.binaryValue,
            keyString.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            identifier: try binaryValue.field(0),
            extendedLanguageTag: try binaryValue.field(1),
            localeIdentifier: try binaryValue.field(2),
            time: try binaryValue.field(3),
            duration: try binaryValue.field(4),
            dataType: try binaryValue.field(5),
            stringValue: try binaryValue.field(6),
            numberValue: try binaryValue.field(7),
            valueDescription: try binaryValue.field(8),
            startDate: try binaryValue.field(9),
            keySpace: try binaryValue.field(10),
            keyString: try binaryValue.field(11)
        )
    }
}
