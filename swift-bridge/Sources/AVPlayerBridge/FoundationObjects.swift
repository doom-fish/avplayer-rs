import AVFoundation
import Foundation

@_cdecl("av_ns_object_release")
public func av_ns_object_release(_ objectPtr: UnsafeMutableRawPointer?) {
    guard let objectPtr else { return }
    Unmanaged<AnyObject>.fromOpaque(objectPtr).release()
}

func avpRetained(_ object: AnyObject) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

func avpDate(fromISO8601 string: String?) -> Date? {
    guard let string else { return nil }
    let formatter = ISO8601DateFormatter()
    return formatter.date(from: string)
}

func avpMediaType(from raw: String) -> AVMediaType {
    switch raw {
    case "audio":
        return .audio
    case "video":
        return .video
    case "text":
        return .text
    case "subtitle":
        return .subtitle
    case "closed_caption":
        return .closedCaption
    case "metadata":
        return .metadata
    case "timecode":
        return .timecode
    case "muxed":
        return .muxed
    case "depth_data":
        return .depthData
    default:
        return AVMediaType(rawValue: raw)
    }
}

func avpMetadataItem(from payload: MetadataItemPayload) -> AVMetadataItem {
    let item = AVMutableMetadataItem()
    if let identifier = payload.identifier {
        item.identifier = AVMetadataIdentifier(rawValue: identifier)
    }
    if let keySpace = payload.keySpace {
        item.keySpace = AVMetadataKeySpace(rawValue: keySpace)
    }
    if let commonKey = payload.commonKey {
        item.key = commonKey as NSString
    }
    if let dataType = payload.dataType {
        item.dataType = dataType
    }
    if let stringValue = payload.stringValue {
        item.value = stringValue as NSString
    } else if let numberValue = payload.numberValue {
        item.value = NSNumber(value: numberValue)
    } else if let valueDescription = payload.valueDescription {
        item.value = valueDescription as NSString
    }
    return item
}

func avpMetadataItems(from payloads: [MetadataItemPayload]) -> [AVMetadataItem] {
    payloads.map(avpMetadataItem)
}
