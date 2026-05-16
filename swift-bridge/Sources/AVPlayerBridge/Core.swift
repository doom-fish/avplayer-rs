import AVFoundation
import CoreMedia
import CoreVideo
import Foundation

let AVP_OK: Int32 = 0
let AVP_INVALID_ARGUMENT: Int32 = -1
let AVP_ASSET_CREATE_FAILED: Int32 = -2
let AVP_PLAYER_CREATE_FAILED: Int32 = -3
let AVP_READER_CREATE_FAILED: Int32 = -4
let AVP_OPERATION_FAILED: Int32 = -5
let AVP_OBSERVER_FAILED: Int32 = -6
let AVP_LOAD_FAILED: Int32 = -7

@_cdecl("avp_string_free")
public func avp_string_free(_ str: UnsafeMutablePointer<CChar>?) {
    guard let str else { return }
    free(str)
}

func ffiString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

enum BridgeError: LocalizedError {
    case message(String)

    var errorDescription: String? {
        switch self {
        case .message(let message):
            return message
        }
    }
}

struct TimePayload: Codable {
    let kind: String
    let value: Int64?
    let timescale: Int32?
}

struct TimeRangePayload: Codable {
    let start: TimePayload
    let duration: TimePayload
}

struct SizePayload: Codable {
    let width: Double
    let height: Double
}

struct MetadataItemPayload: Codable {
    let identifier: String?
    let keySpace: String?
    let commonKey: String?
    let stringValue: String?
    let numberValue: Double?
    let dataType: String?
    let valueDescription: String?
}

struct AssetInfoPayload: Codable {
    let url: String?
    let duration: TimePayload
    let metadata: [MetadataItemPayload]
}

struct TrackInfoPayload: Codable {
    let trackId: Int32
    let mediaType: String
    let naturalSize: SizePayload
    let nominalFrameRate: String
    let estimatedDataRate: String
}

struct KeyLoadStatusPayload: Codable {
    let key: String
    let status: Int32
    let errorMessage: String?
}

struct PlayerInfoPayload: Codable {
    let status: Int32
    let errorMessage: String?
    let rate: Float
    let currentTime: TimePayload
    let duration: TimePayload
}

struct PlayerItemInfoPayload: Codable {
    let status: Int32
    let errorMessage: String?
    let duration: TimePayload
    let presentationSize: SizePayload
    let metadata: [MetadataItemPayload]
}

struct PlayerItemEventPayload: Codable {
    let event: String
    let status: Int32?
    let errorMessage: String?
    let presentationSize: SizePayload?
}

struct ReaderInfoPayload: Codable {
    let status: Int32
    let errorMessage: String?
    let timeRange: TimeRangePayload
    let outputCount: Int
}

struct VideoOutputSettingsPayload: Codable {
    let pixelFormat: UInt32
    let width: Int?
    let height: Int?
}

struct AudioOutputSettingsPayload: Codable {
    let sampleRate: Double?
    let channelCount: UInt32?
    let bitsPerChannel: UInt32
    let isFloat: Bool
    let isNonInterleaved: Bool
}

func avpEncodeJSON<T: Encodable>(_ value: T) throws -> String {
    let data = try JSONEncoder().encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw BridgeError.message("failed to UTF-8 encode JSON payload")
    }
    return string
}

func avpDecodeJSON<T: Decodable>(_ ptr: UnsafePointer<CChar>?, as type: T.Type) throws -> T {
    guard let ptr else {
        throw BridgeError.message("missing JSON payload")
    }
    let string = String(cString: ptr)
    guard let data = string.data(using: .utf8) else {
        throw BridgeError.message("payload was not valid UTF-8")
    }
    return try JSONDecoder().decode(T.self, from: data)
}

func avpBlockOnAsync<T>(
    timeoutSeconds: Int = 30,
    work: @escaping () async throws -> T,
    onSuccess: @escaping (T) -> Void,
    outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let semaphore = DispatchSemaphore(value: 0)
    var status = AVP_OK
    var finished = false

    Task {
        defer {
            finished = true
            semaphore.signal()
        }
        do {
            let result = try await work()
            onSuccess(result)
        } catch {
            status = AVP_LOAD_FAILED
            outErrorMessage?.pointee = ffiString(error.localizedDescription)
        }
    }

    if semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .timedOut, !finished {
        outErrorMessage?.pointee = ffiString("timed out waiting for async bridge operation")
        return AVP_LOAD_FAILED
    }

    return status
}

func cmTime(from payload: TimePayload) -> CMTime {
    switch payload.kind {
    case "numeric":
        return CMTime(value: payload.value ?? 0, timescale: payload.timescale ?? 1)
    case "invalid":
        return .invalid
    case "indefinite":
        return .indefinite
    case "positive_infinity":
        return .positiveInfinity
    case "negative_infinity":
        return .negativeInfinity
    default:
        return .invalid
    }
}

func cmTime(value: Int64, timescale: Int32, kind: Int32) -> CMTime {
    switch kind {
    case 0:
        return CMTime(value: value, timescale: timescale)
    case 1:
        return .invalid
    case 2:
        return .indefinite
    case 3:
        return .positiveInfinity
    case 4:
        return .negativeInfinity
    default:
        return .invalid
    }
}

func encodeTime(_ time: CMTime) -> TimePayload {
    if time == .invalid {
        return TimePayload(kind: "invalid", value: nil, timescale: nil)
    }
    if time == .indefinite {
        return TimePayload(kind: "indefinite", value: nil, timescale: nil)
    }
    if time == .positiveInfinity {
        return TimePayload(kind: "positive_infinity", value: nil, timescale: nil)
    }
    if time == .negativeInfinity {
        return TimePayload(kind: "negative_infinity", value: nil, timescale: nil)
    }
    return TimePayload(kind: "numeric", value: time.value, timescale: time.timescale)
}

func cmTimeRange(from payload: TimeRangePayload) -> CMTimeRange {
    CMTimeRange(start: cmTime(from: payload.start), duration: cmTime(from: payload.duration))
}

func encodeTimeRange(_ range: CMTimeRange) -> TimeRangePayload {
    TimeRangePayload(start: encodeTime(range.start), duration: encodeTime(range.duration))
}

func encodeSize(_ size: CGSize) -> SizePayload {
    SizePayload(width: size.width, height: size.height)
}

func avpEncodeMetadataItem(_ item: AVMetadataItem) -> MetadataItemPayload {
    MetadataItemPayload(
        identifier: item.identifier?.rawValue,
        keySpace: item.keySpace?.rawValue,
        commonKey: item.commonKey?.rawValue,
        stringValue: item.stringValue,
        numberValue: item.numberValue?.doubleValue,
        dataType: item.dataType,
        valueDescription: item.value.map(String.init(describing:))
    )
}

func avpMediaTypeString(_ mediaType: AVMediaType) -> String {
    switch mediaType {
    case .audio:
        return "audio"
    case .video:
        return "video"
    case .text:
        return "text"
    case .subtitle:
        return "subtitle"
    case .closedCaption:
        return "closed_caption"
    case .metadata:
        return "metadata"
    case .timecode:
        return "timecode"
    case .muxed:
        return "muxed"
    case .depthData:
        return "depth_data"
    default:
        return mediaType.rawValue
    }
}

func avpDispatchQueue(from labelPtr: UnsafePointer<CChar>?) -> DispatchQueue? {
    guard let labelPtr else { return nil }
    return DispatchQueue(label: String(cString: labelPtr))
}
