import AVFoundation
import CoreGraphics
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
let AVP_TIMED_OUT: Int32 = -8

public typealias AVPJsonCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?
) -> Void
public typealias AVPBoolJsonCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?
) -> Bool
public typealias AVPPeriodicTimeCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int64,
    Int32,
    Int32
) -> Void
public typealias AVPSimpleCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias AVPBoolObjectCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?,
    UnsafeMutableRawPointer?
) -> Bool
public typealias AVPDropCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias AVPJsonObjectsCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?,
    UnsafePointer<UnsafeMutableRawPointer?>?,
    Int
) -> Void

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

struct RectPayload: Codable {
    let x: Double
    let y: Double
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
    let timeControlStatus: Int32?
    let reasonForWaitingToPlay: String?
    let actionAtItemEnd: Int32?
    let volume: Float?
    let muted: Bool?
    let automaticallyWaitsToMinimizeStalling: Bool?
    let appliesMediaSelectionCriteriaAutomatically: Bool?
    let eligibleForHdrPlayback: Bool?
    let audiovisualBackgroundPlaybackPolicy: Int32?
    let networkResourcePriority: Int32?
    let defaultRate: Float?
    let audioOutputDeviceUniqueId: String?
}

struct PlayerItemVideoCompositorPayload: Codable {
    let className: String
    let supportsWideColorSourceFrames: Bool?
    let supportsHdrSourceFrames: Bool?
    let supportsSourceTaggedBuffers: Bool?
    let canConformColorOfSourceFrames: Bool?
}

struct PlayerItemInfoPayload: Codable {
    let status: Int32
    let errorMessage: String?
    let duration: TimePayload
    let presentationSize: SizePayload
    let metadata: [MetadataItemPayload]
    let automaticallyLoadedAssetKeys: [String]
    let seekableTimeRanges: [TimeRangePayload]
    let loadedTimeRanges: [TimeRangePayload]
    let canUseNetworkResourcesForLiveStreamingWhilePaused: Bool
    let preferredForwardBufferDuration: Double
    let preferredPeakBitRate: Double
    let preferredPeakBitRateForExpensiveNetworks: Double
    let preferredMaximumResolution: SizePayload
    let preferredMaximumResolutionForExpensiveNetworks: SizePayload
    let audioTimePitchAlgorithm: String
    let outputCount: Int
    let trackCount: Int
    let variantPreferences: UInt64?
    let authorizationRequiredForPlayback: Bool
    let applicationAuthorizedForPlayback: Bool
    let contentAuthorizedForPlayback: Bool
    let contentAuthorizationRequestStatus: Int32
    let customVideoCompositor: PlayerItemVideoCompositorPayload?
    let forwardPlaybackEndTime: TimePayload
    let reversePlaybackEndTime: TimePayload
    let canStepForward: Bool
    let canStepBackward: Bool
    let hasVideoComposition: Bool
    let hasAudioMix: Bool
}

struct PlayerItemEventPayload: Codable {
    let event: String
    let status: Int32?
    let errorMessage: String?
    let presentationSize: SizePayload?
    let hasOriginatingParticipant: Bool?
    let recommendedTimeOffsetFromLive: TimePayload?
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

struct PlayerItemTrackInfoPayload: Codable {
    let enabled: Bool
    let currentVideoFrameRate: Float
    let videoFieldMode: String?
    let hasAssetTrack: Bool
}

struct AccessLogPayload: Codable {
    let extendedLog: String?
    let extendedLogDataStringEncoding: UInt
    let events: [AccessLogEventPayload]
}

struct AccessLogEventPayload: Codable {
    let numberOfMediaRequests: Int
    let playbackStartDate: String?
    let uri: String?
    let serverAddress: String?
    let numberOfServerAddressChanges: Int
    let playbackSessionId: String?
    let playbackStartOffset: Double
    let segmentsDownloadedDuration: Double
    let durationWatched: Double
    let numberOfStalls: Int
    let numberOfBytesTransferred: Int64
    let transferDuration: Double
    let observedBitrate: Double
    let indicatedBitrate: Double
    let indicatedAverageBitrate: Double
    let averageVideoBitrate: Double
    let averageAudioBitrate: Double
    let numberOfDroppedVideoFrames: Int
    let startupTime: Double
    let downloadOverdue: Int
    let observedBitrateStandardDeviation: Double
    let playbackType: String?
    let mediaRequestsWwan: Int
    let switchBitrate: Double
}

struct ErrorLogPayload: Codable {
    let extendedLog: String?
    let extendedLogDataStringEncoding: UInt
    let events: [ErrorLogEventPayload]
}

struct ErrorLogEventPayload: Codable {
    let date: String?
    let uri: String?
    let serverAddress: String?
    let playbackSessionId: String?
    let errorStatusCode: Int
    let errorDomain: String
    let errorComment: String?
    let allHttpResponseHeaderFields: [String: String]?
}

struct PlayerLayerInfoPayload: Codable {
    let hasPlayer: Bool
    let videoGravity: String
    let readyForDisplay: Bool
    let videoRect: RectPayload
}

struct PlayerLooperInfoPayload: Codable {
    let status: Int32
    let errorMessage: String?
    let loopCount: Int
    let loopingItemCount: Int
}

struct MediaSelectionCriteriaPayload: Codable {
    let preferredLanguages: [String]?
    let preferredMediaCharacteristics: [String]?
    let principalMediaCharacteristics: [String]?
}

struct VideoOutputInfoPayload: Codable {
    let suppressesPlayerRendering: Bool
    let hasDelegate: Bool
}

struct MetadataOutputInfoPayload: Codable {
    let suppressesPlayerRendering: Bool
    let advanceIntervalForDelegateInvocation: Double
    let identifiers: [String]?
    let hasDelegate: Bool
}

struct LegibleOutputInfoPayload: Codable {
    let suppressesPlayerRendering: Bool
    let advanceIntervalForDelegateInvocation: Double
    let nativeRepresentationSubtypes: [UInt32]
    let hasDelegate: Bool
    let textStylingResolution: String
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

struct AVPAwaitError: Error {
    let status: Int32
    let message: String
}

private final class AVPAwaitState<T>: @unchecked Sendable {
    private let lock = NSLock()
    private var result: Result<T, Error>?

    func store(_ value: Result<T, Error>) {
        lock.lock()
        result = value
        lock.unlock()
    }

    func take() -> Result<T, Error>? {
        lock.lock()
        defer { lock.unlock() }
        let value = result
        result = nil
        return value
    }
}

func avpAwait<T>(
    label: String,
    timeoutSeconds: Int = 30,
    work: @escaping () async throws -> T
) -> Result<T, AVPAwaitError> {
    let semaphore = DispatchSemaphore(value: 0)
    let state = AVPAwaitState<T>()
    let task = Task {
        do {
            state.store(.success(try await work()))
        } catch {
            state.store(.failure(error))
        }
        semaphore.signal()
    }

    if semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .timedOut {
        task.cancel()
        return .failure(AVPAwaitError(status: AVP_TIMED_OUT, message: "timed out waiting for \(label)"))
    }

    switch state.take() {
    case .success(let value):
        return .success(value)
    case .failure(let error):
        return .failure(AVPAwaitError(status: AVP_LOAD_FAILED, message: error.localizedDescription))
    case nil:
        return .failure(AVPAwaitError(status: AVP_OPERATION_FAILED, message: "missing result for \(label)"))
    }
}

func avpWriteError(
    _ error: AVPAwaitError,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) {
    outErrorMessage?.pointee = ffiString(error.message)
}

final class AVPCallbackGate: @unchecked Sendable {
    let context: UnsafeMutableRawPointer?
    private let release: AVPDropCallback?
    private let lock = NSLock()
    private var closed = false
    private var active = 0
    private var released = false

    init(context: UnsafeMutableRawPointer?, release: AVPDropCallback?) {
        self.context = context
        self.release = release
    }

    deinit {
        if !released, let context, let release {
            release(context)
        }
    }

    func enter() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        guard !closed else { return false }
        active += 1
        return true
    }

    func leave() {
        lock.lock()
        active -= 1
        let shouldRelease = closed && active == 0 && !released
        if shouldRelease {
            released = true
        }
        lock.unlock()
        if shouldRelease {
            releaseContext()
        }
    }

    func close() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        guard !closed else { return false }
        closed = true
        return true
    }

    var isBusy: Bool {
        lock.lock()
        defer { lock.unlock() }
        return active > 0
    }

    func finish() {
        lock.lock()
        let shouldRelease = closed && active == 0 && !released
        if shouldRelease {
            released = true
        }
        lock.unlock()
        if shouldRelease {
            releaseContext()
        }
    }

    func run<R>(_ fallback: R, _ body: (UnsafeMutableRawPointer?) -> R) -> R {
        guard enter() else { return fallback }
        defer { leave() }
        return body(context)
    }

    func deliverJSON<T: Encodable>(_ payload: T, to callback: AVPJsonCallback) {
        run(()) { context in
            guard let json = try? avpEncodeJSON(payload) else {
                callback(context, nil)
                return
            }
            json.withCString { callback(context, $0) }
        }
    }

    func deliverJSON<T: Encodable>(_ payload: T, objects: [AnyObject], to callback: AVPJsonObjectsCallback) {
        run(()) { context in
            guard let json = try? avpEncodeJSON(payload) else {
                callback(context, nil, nil, 0)
                return
            }
            let pointers: [UnsafeMutableRawPointer?] = objects.map { Unmanaged.passUnretained($0).toOpaque() }
            withExtendedLifetime(objects) {
                json.withCString { cJSON in
                    pointers.withUnsafeBufferPointer { buffer in
                        callback(context, cJSON, buffer.baseAddress, buffer.count)
                    }
                }
            }
        }
    }

    private func releaseContext() {
        if let context, let release {
            release(context)
        }
    }
}

final class AVPOnceFlag: @unchecked Sendable {
    private let lock = NSLock()
    private var fired = false

    func claim() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        guard !fired else { return false }
        fired = true
        return true
    }
}

final class AVPQueueToken {}

private let avpQueueKey = DispatchSpecificKey<AVPQueueToken>()

func avpIsCurrentQueue(_ queue: DispatchQueue?) -> Bool {
    guard let queue else { return Thread.isMainThread }
    if queue === DispatchQueue.main {
        return Thread.isMainThread
    }
    guard let token = queue.getSpecific(key: avpQueueKey) else { return false }
    return DispatchQueue.getSpecific(key: avpQueueKey) === token
}

func avpSeekTimeError(_ time: CMTime, what: String) -> String? {
    if !time.isValid {
        return "\(what) must be a valid time"
    }
    if time.isIndefinite {
        return "\(what) must not be indefinite"
    }
    return nil
}

func avpToleranceError(_ tolerance: CMTime, what: String) -> String? {
    if !tolerance.isValid || tolerance.isIndefinite {
        return "\(what) must be a valid time"
    }
    if tolerance.isNegativeInfinity || (tolerance.isNumeric && tolerance.value < 0) {
        return "\(what) must not be negative"
    }
    return nil
}

final class AVPObjectArrayBox: NSObject {
    let objects: [AnyObject]

    init(objects: [AnyObject]) {
        self.objects = objects
    }
}

@_cdecl("avp_object_array_count")
public func avp_object_array_count(_ arrayPtr: UnsafeMutableRawPointer) -> Int {
    Unmanaged<AVPObjectArrayBox>.fromOpaque(arrayPtr).takeUnretainedValue().objects.count
}

@_cdecl("avp_object_array_copy_at")
public func avp_object_array_copy_at(
    _ arrayPtr: UnsafeMutableRawPointer,
    _ index: Int
) -> UnsafeMutableRawPointer? {
    let objects = Unmanaged<AVPObjectArrayBox>.fromOpaque(arrayPtr).takeUnretainedValue().objects
    guard index >= 0, index < objects.count, !(objects[index] is NSNull) else { return nil }
    return Unmanaged.passRetained(objects[index]).toOpaque()
}

@_cdecl("avp_object_array_release")
public func avp_object_array_release(_ arrayPtr: UnsafeMutableRawPointer?) {
    guard let arrayPtr else { return }
    Unmanaged<AVPObjectArrayBox>.fromOpaque(arrayPtr).release()
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

func encodeTimeRanges(_ values: [NSValue]) -> [TimeRangePayload] {
    values.map { encodeTimeRange($0.timeRangeValue) }
}

func encodeSize(_ size: CGSize) -> SizePayload {
    SizePayload(width: size.width, height: size.height)
}

func encodeRect(_ rect: CGRect) -> RectPayload {
    RectPayload(x: rect.origin.x, y: rect.origin.y, width: rect.size.width, height: rect.size.height)
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

func avpMediaCharacteristicString(_ characteristic: AVMediaCharacteristic) -> String {
    switch characteristic {
    case .audible:
        return "audible"
    case .legible:
        return "legible"
    case .visual:
        return "visual"
    case .containsOnlyForcedSubtitles:
        return "contains_only_forced_subtitles"
    case .transcribesSpokenDialogForAccessibility:
        return "transcribes_spoken_dialog_for_accessibility"
    case .describesMusicAndSoundForAccessibility:
        return "describes_music_and_sound_for_accessibility"
    case .describesVideoForAccessibility:
        return "describes_video_for_accessibility"
    case .easyToRead:
        return "easy_to_read"
    case .languageTranslation:
        return "language_translation"
    case .dubbedTranslation:
        return "dubbed_translation"
    case .voiceOverTranslation:
        return "voice_over_translation"
    case .isOriginalContent:
        return "is_original_content"
    default:
        return characteristic.rawValue
    }
}

func avpMediaCharacteristic(from raw: String) -> AVMediaCharacteristic {
    switch raw {
    case "audible":
        return .audible
    case "legible":
        return .legible
    case "visual":
        return .visual
    case "contains_only_forced_subtitles":
        return .containsOnlyForcedSubtitles
    case "transcribes_spoken_dialog_for_accessibility":
        return .transcribesSpokenDialogForAccessibility
    case "describes_music_and_sound_for_accessibility":
        return .describesMusicAndSoundForAccessibility
    case "describes_video_for_accessibility":
        return .describesVideoForAccessibility
    case "easy_to_read":
        return .easyToRead
    case "language_translation":
        return .languageTranslation
    case "dubbed_translation":
        return .dubbedTranslation
    case "voice_over_translation":
        return .voiceOverTranslation
    case "is_original_content":
        return .isOriginalContent
    default:
        return AVMediaCharacteristic(rawValue: raw)
    }
}

func avpAudioTimePitchAlgorithmString(_ algorithm: AVAudioTimePitchAlgorithm) -> String {
    switch algorithm {
    case .spectral:
        return "spectral"
    case .timeDomain:
        return "time_domain"
    case .varispeed:
        return "varispeed"
    default:
        if algorithm.rawValue == "AVAudioTimePitchAlgorithmLowQualityZeroLatency" {
            return "low_quality_zero_latency"
        }
        return algorithm.rawValue
    }
}

func avpAudioTimePitchAlgorithm(from raw: String) -> AVAudioTimePitchAlgorithm {
    switch raw {
    case "spectral":
        return .spectral
    case "time_domain":
        return .timeDomain
    case "varispeed":
        return .varispeed
    case "low_quality_zero_latency":
        return AVAudioTimePitchAlgorithm(rawValue: "AVAudioTimePitchAlgorithmLowQualityZeroLatency")
    default:
        return AVAudioTimePitchAlgorithm(rawValue: raw)
    }
}

func avpVideoGravityString(_ gravity: AVLayerVideoGravity) -> String {
    switch gravity {
    case .resizeAspect:
        return "resize_aspect"
    case .resizeAspectFill:
        return "resize_aspect_fill"
    case .resize:
        return "resize"
    default:
        return gravity.rawValue
    }
}

func avpVideoGravity(from raw: String) -> AVLayerVideoGravity {
    switch raw {
    case "resize_aspect":
        return .resizeAspect
    case "resize_aspect_fill":
        return .resizeAspectFill
    case "resize":
        return .resize
    default:
        return AVLayerVideoGravity(rawValue: raw)
    }
}

func avpISO8601String(_ date: Date?) -> String? {
    guard let date else { return nil }
    return ISO8601DateFormatter().string(from: date)
}

func avpString(from data: Data?, encoding: UInt) -> String? {
    guard let data else { return nil }
    return String(data: data, encoding: String.Encoding(rawValue: encoding))
}

func avpDispatchQueue(from labelPtr: UnsafePointer<CChar>?) -> DispatchQueue? {
    guard let labelPtr else { return nil }
    let queue = DispatchQueue(label: String(cString: labelPtr))
    queue.setSpecific(key: avpQueueKey, value: AVPQueueToken())
    return queue
}
