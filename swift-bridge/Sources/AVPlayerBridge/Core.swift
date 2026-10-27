import AVPlayerABI
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

public typealias AVPValueCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<AVPValue>?
) -> Void
public typealias AVPBoolValueCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<AVPValue>?
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
    UnsafePointer<AVPValue>?,
    UnsafeMutableRawPointer?
) -> Bool
public typealias AVPDropCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias AVPValueObjectsCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<AVPValue>?,
    UnsafePointer<UnsafeMutableRawPointer?>?,
    Int
) -> Void



func ffiString(_ string: String) -> UnsafeMutablePointer<AVPValue>? {
    try? avpEncodeValue(string)
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

struct TimePayload: AVPBridge {
    let kind: Int32
    let value: Int64?
    let timescale: Int32?
}

struct TimeRangePayload: AVPBridge {
    let start: TimePayload
    let duration: TimePayload
}

struct SizePayload: AVPBridge {
    let width: Double
    let height: Double
}

struct RectPayload: AVPBridge {
    let x: Double
    let y: Double
    let width: Double
    let height: Double
}

struct MetadataItemPayload: AVPBridge {
    let identifier: String?
    let keySpace: String?
    let commonKey: String?
    let stringValue: String?
    let numberValue: Double?
    let dataType: String?
    let valueDescription: String?
}

struct AssetInfoPayload: AVPBridge {
    let url: String?
    let duration: TimePayload
    let metadata: [MetadataItemPayload]
}

struct TrackInfoPayload: AVPBridge {
    let trackId: Int32
    let mediaType: String
    let naturalSize: SizePayload
    let nominalFrameRate: Float
    let estimatedDataRate: Float
}

struct KeyLoadStatusPayload: AVPBridge {
    let key: String
    let status: Int32
    let errorMessage: String?
}

struct PlayerInfoPayload: AVPBridge {
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

struct PlayerItemVideoCompositorPayload: AVPBridge {
    let className: String
    let supportsWideColorSourceFrames: Bool?
    let supportsHdrSourceFrames: Bool?
    let supportsSourceTaggedBuffers: Bool?
    let canConformColorOfSourceFrames: Bool?
}

struct PlayerItemInfoPayload: AVPBridge {
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

struct PlayerItemEventPayload: AVPBridge {
    let event: String
    let status: Int32?
    let errorMessage: String?
    let presentationSize: SizePayload?
    let hasOriginatingParticipant: Bool?
    let recommendedTimeOffsetFromLive: TimePayload?
}

struct ReaderInfoPayload: AVPBridge {
    let status: Int32
    let errorMessage: String?
    let timeRange: TimeRangePayload
    let outputCount: Int
}

struct VideoOutputSettingsPayload: AVPBridge {
    let pixelFormat: UInt32
    let width: Int?
    let height: Int?
}

struct AudioOutputSettingsPayload: AVPBridge {
    let sampleRate: Double?
    let channelCount: UInt32?
    let bitsPerChannel: UInt32
    let isFloat: Bool
    let isNonInterleaved: Bool
}

struct PlayerItemTrackInfoPayload: AVPBridge {
    let enabled: Bool
    let currentVideoFrameRate: Float
    let videoFieldMode: String?
    let hasAssetTrack: Bool
}

struct AccessLogPayload: AVPBridge {
    let extendedLog: String?
    let extendedLogDataStringEncoding: UInt
    let events: [AccessLogEventPayload]
}

struct AccessLogEventPayload: AVPBridge {
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

struct ErrorLogPayload: AVPBridge {
    let extendedLog: String?
    let extendedLogDataStringEncoding: UInt
    let events: [ErrorLogEventPayload]
}

struct ErrorLogEventPayload: AVPBridge {
    let date: String?
    let uri: String?
    let serverAddress: String?
    let playbackSessionId: String?
    let errorStatusCode: Int
    let errorDomain: String
    let errorComment: String?
    let allHttpResponseHeaderFields: [String: String]?
}

struct PlayerLayerInfoPayload: AVPBridge {
    let hasPlayer: Bool
    let videoGravity: String
    let readyForDisplay: Bool
    let videoRect: RectPayload
}

struct PlayerLooperInfoPayload: AVPBridge {
    let status: Int32
    let errorMessage: String?
    let loopCount: Int
    let loopingItemCount: Int
}

struct MediaSelectionCriteriaPayload: AVPBridge {
    let preferredLanguages: [String]?
    let preferredMediaCharacteristics: [String]?
    let principalMediaCharacteristics: [String]?
}

struct VideoOutputInfoPayload: AVPBridge {
    let suppressesPlayerRendering: Bool
    let hasDelegate: Bool
}

struct MetadataOutputInfoPayload: AVPBridge {
    let suppressesPlayerRendering: Bool
    let advanceIntervalForDelegateInvocation: Double
    let identifiers: [String]?
    let hasDelegate: Bool
}

struct LegibleOutputInfoPayload: AVPBridge {
    let suppressesPlayerRendering: Bool
    let advanceIntervalForDelegateInvocation: Double
    let nativeRepresentationSubtypes: [UInt32]
    let hasDelegate: Bool
    let textStylingResolution: String
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
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
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

    func deliverValue<T: AVPBridge>(_ payload: T, to callback: AVPValueCallback) {
        run(()) { context in
            guard let value = try? avpEncodeValue(payload) else {
                callback(context, nil)
                return
            }
            defer { avp_value_free(value) }
            callback(context, UnsafePointer(value))
        }
    }

    func deliverValue<T: AVPBridge>(_ payload: T, objects: [AnyObject], to callback: AVPValueObjectsCallback) {
        run(()) { context in
            guard let value = try? avpEncodeValue(payload) else {
                callback(context, nil, nil, 0)
                return
            }
            defer { avp_value_free(value) }
            let pointers: [UnsafeMutableRawPointer?] = objects.map { Unmanaged.passUnretained($0).toOpaque() }
            withExtendedLifetime(objects) {
                pointers.withUnsafeBufferPointer { buffer in
                    callback(context, UnsafePointer(value), buffer.baseAddress, buffer.count)
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
    case 0:
        return CMTime(value: payload.value ?? 0, timescale: payload.timescale ?? 1)
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
        return TimePayload(kind: 1, value: nil, timescale: nil)
    }
    if time == .indefinite {
        return TimePayload(kind: 2, value: nil, timescale: nil)
    }
    if time == .positiveInfinity {
        return TimePayload(kind: 3, value: nil, timescale: nil)
    }
    if time == .negativeInfinity {
        return TimePayload(kind: 4, value: nil, timescale: nil)
    }
    return TimePayload(kind: 0, value: time.value, timescale: time.timescale)
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

func avpDispatchQueue(from labelPtr: UnsafePointer<AVPValue>?) -> DispatchQueue? {
    guard let labelPtr else { return nil }
    let queue = DispatchQueue(label: String(bridge: labelPtr))
    queue.setSpecific(key: avpQueueKey, value: AVPQueueToken())
    return queue
}

extension TimePayload {
    var binaryValue: AVPBinaryValue {
        .array([
            kind.binaryValue,
            value.binaryValue,
            timescale.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            kind: try binaryValue.field(0),
            value: try binaryValue.field(1),
            timescale: try binaryValue.field(2)
        )
    }
}

extension TimeRangePayload {
    var binaryValue: AVPBinaryValue {
        .array([
            start.binaryValue,
            duration.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            start: try binaryValue.field(0),
            duration: try binaryValue.field(1)
        )
    }
}

extension SizePayload {
    var binaryValue: AVPBinaryValue {
        .array([
            width.binaryValue,
            height.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            width: try binaryValue.field(0),
            height: try binaryValue.field(1)
        )
    }
}

extension RectPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            x.binaryValue,
            y.binaryValue,
            width.binaryValue,
            height.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            x: try binaryValue.field(0),
            y: try binaryValue.field(1),
            width: try binaryValue.field(2),
            height: try binaryValue.field(3)
        )
    }
}

extension MetadataItemPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            identifier.binaryValue,
            keySpace.binaryValue,
            commonKey.binaryValue,
            stringValue.binaryValue,
            numberValue.binaryValue,
            dataType.binaryValue,
            valueDescription.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            identifier: try binaryValue.field(0),
            keySpace: try binaryValue.field(1),
            commonKey: try binaryValue.field(2),
            stringValue: try binaryValue.field(3),
            numberValue: try binaryValue.field(4),
            dataType: try binaryValue.field(5),
            valueDescription: try binaryValue.field(6)
        )
    }
}

extension AssetInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            url.binaryValue,
            duration.binaryValue,
            metadata.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            url: try binaryValue.field(0),
            duration: try binaryValue.field(1),
            metadata: try binaryValue.field(2)
        )
    }
}

extension TrackInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            trackId.binaryValue,
            mediaType.binaryValue,
            naturalSize.binaryValue,
            nominalFrameRate.binaryValue,
            estimatedDataRate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            trackId: try binaryValue.field(0),
            mediaType: try binaryValue.field(1),
            naturalSize: try binaryValue.field(2),
            nominalFrameRate: try binaryValue.field(3),
            estimatedDataRate: try binaryValue.field(4)
        )
    }
}

extension KeyLoadStatusPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            key.binaryValue,
            status.binaryValue,
            errorMessage.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            key: try binaryValue.field(0),
            status: try binaryValue.field(1),
            errorMessage: try binaryValue.field(2)
        )
    }
}

extension PlayerInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            status.binaryValue,
            errorMessage.binaryValue,
            rate.binaryValue,
            currentTime.binaryValue,
            duration.binaryValue,
            timeControlStatus.binaryValue,
            reasonForWaitingToPlay.binaryValue,
            actionAtItemEnd.binaryValue,
            volume.binaryValue,
            muted.binaryValue,
            automaticallyWaitsToMinimizeStalling.binaryValue,
            appliesMediaSelectionCriteriaAutomatically.binaryValue,
            eligibleForHdrPlayback.binaryValue,
            audiovisualBackgroundPlaybackPolicy.binaryValue,
            networkResourcePriority.binaryValue,
            defaultRate.binaryValue,
            audioOutputDeviceUniqueId.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            status: try binaryValue.field(0),
            errorMessage: try binaryValue.field(1),
            rate: try binaryValue.field(2),
            currentTime: try binaryValue.field(3),
            duration: try binaryValue.field(4),
            timeControlStatus: try binaryValue.field(5),
            reasonForWaitingToPlay: try binaryValue.field(6),
            actionAtItemEnd: try binaryValue.field(7),
            volume: try binaryValue.field(8),
            muted: try binaryValue.field(9),
            automaticallyWaitsToMinimizeStalling: try binaryValue.field(10),
            appliesMediaSelectionCriteriaAutomatically: try binaryValue.field(11),
            eligibleForHdrPlayback: try binaryValue.field(12),
            audiovisualBackgroundPlaybackPolicy: try binaryValue.field(13),
            networkResourcePriority: try binaryValue.field(14),
            defaultRate: try binaryValue.field(15),
            audioOutputDeviceUniqueId: try binaryValue.field(16)
        )
    }
}

extension PlayerItemVideoCompositorPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            className.binaryValue,
            supportsWideColorSourceFrames.binaryValue,
            supportsHdrSourceFrames.binaryValue,
            supportsSourceTaggedBuffers.binaryValue,
            canConformColorOfSourceFrames.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            className: try binaryValue.field(0),
            supportsWideColorSourceFrames: try binaryValue.field(1),
            supportsHdrSourceFrames: try binaryValue.field(2),
            supportsSourceTaggedBuffers: try binaryValue.field(3),
            canConformColorOfSourceFrames: try binaryValue.field(4)
        )
    }
}

extension PlayerItemInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            status.binaryValue,
            errorMessage.binaryValue,
            duration.binaryValue,
            presentationSize.binaryValue,
            metadata.binaryValue,
            automaticallyLoadedAssetKeys.binaryValue,
            seekableTimeRanges.binaryValue,
            loadedTimeRanges.binaryValue,
            canUseNetworkResourcesForLiveStreamingWhilePaused.binaryValue,
            preferredForwardBufferDuration.binaryValue,
            preferredPeakBitRate.binaryValue,
            preferredPeakBitRateForExpensiveNetworks.binaryValue,
            preferredMaximumResolution.binaryValue,
            preferredMaximumResolutionForExpensiveNetworks.binaryValue,
            audioTimePitchAlgorithm.binaryValue,
            outputCount.binaryValue,
            trackCount.binaryValue,
            variantPreferences.binaryValue,
            authorizationRequiredForPlayback.binaryValue,
            applicationAuthorizedForPlayback.binaryValue,
            contentAuthorizedForPlayback.binaryValue,
            contentAuthorizationRequestStatus.binaryValue,
            customVideoCompositor.binaryValue,
            forwardPlaybackEndTime.binaryValue,
            reversePlaybackEndTime.binaryValue,
            canStepForward.binaryValue,
            canStepBackward.binaryValue,
            hasVideoComposition.binaryValue,
            hasAudioMix.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            status: try binaryValue.field(0),
            errorMessage: try binaryValue.field(1),
            duration: try binaryValue.field(2),
            presentationSize: try binaryValue.field(3),
            metadata: try binaryValue.field(4),
            automaticallyLoadedAssetKeys: try binaryValue.field(5),
            seekableTimeRanges: try binaryValue.field(6),
            loadedTimeRanges: try binaryValue.field(7),
            canUseNetworkResourcesForLiveStreamingWhilePaused: try binaryValue.field(8),
            preferredForwardBufferDuration: try binaryValue.field(9),
            preferredPeakBitRate: try binaryValue.field(10),
            preferredPeakBitRateForExpensiveNetworks: try binaryValue.field(11),
            preferredMaximumResolution: try binaryValue.field(12),
            preferredMaximumResolutionForExpensiveNetworks: try binaryValue.field(13),
            audioTimePitchAlgorithm: try binaryValue.field(14),
            outputCount: try binaryValue.field(15),
            trackCount: try binaryValue.field(16),
            variantPreferences: try binaryValue.field(17),
            authorizationRequiredForPlayback: try binaryValue.field(18),
            applicationAuthorizedForPlayback: try binaryValue.field(19),
            contentAuthorizedForPlayback: try binaryValue.field(20),
            contentAuthorizationRequestStatus: try binaryValue.field(21),
            customVideoCompositor: try binaryValue.field(22),
            forwardPlaybackEndTime: try binaryValue.field(23),
            reversePlaybackEndTime: try binaryValue.field(24),
            canStepForward: try binaryValue.field(25),
            canStepBackward: try binaryValue.field(26),
            hasVideoComposition: try binaryValue.field(27),
            hasAudioMix: try binaryValue.field(28)
        )
    }
}

extension PlayerItemEventPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            event.binaryValue,
            status.binaryValue,
            errorMessage.binaryValue,
            presentationSize.binaryValue,
            hasOriginatingParticipant.binaryValue,
            recommendedTimeOffsetFromLive.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            event: try binaryValue.field(0),
            status: try binaryValue.field(1),
            errorMessage: try binaryValue.field(2),
            presentationSize: try binaryValue.field(3),
            hasOriginatingParticipant: try binaryValue.field(4),
            recommendedTimeOffsetFromLive: try binaryValue.field(5)
        )
    }
}

extension ReaderInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            status.binaryValue,
            errorMessage.binaryValue,
            timeRange.binaryValue,
            outputCount.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            status: try binaryValue.field(0),
            errorMessage: try binaryValue.field(1),
            timeRange: try binaryValue.field(2),
            outputCount: try binaryValue.field(3)
        )
    }
}

extension VideoOutputSettingsPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            pixelFormat.binaryValue,
            width.binaryValue,
            height.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            pixelFormat: try binaryValue.field(0),
            width: try binaryValue.field(1),
            height: try binaryValue.field(2)
        )
    }
}

extension AudioOutputSettingsPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            sampleRate.binaryValue,
            channelCount.binaryValue,
            bitsPerChannel.binaryValue,
            isFloat.binaryValue,
            isNonInterleaved.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            sampleRate: try binaryValue.field(0),
            channelCount: try binaryValue.field(1),
            bitsPerChannel: try binaryValue.field(2),
            isFloat: try binaryValue.field(3),
            isNonInterleaved: try binaryValue.field(4)
        )
    }
}

extension PlayerItemTrackInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            enabled.binaryValue,
            currentVideoFrameRate.binaryValue,
            videoFieldMode.binaryValue,
            hasAssetTrack.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            enabled: try binaryValue.field(0),
            currentVideoFrameRate: try binaryValue.field(1),
            videoFieldMode: try binaryValue.field(2),
            hasAssetTrack: try binaryValue.field(3)
        )
    }
}

extension AccessLogPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            extendedLog.binaryValue,
            extendedLogDataStringEncoding.binaryValue,
            events.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            extendedLog: try binaryValue.field(0),
            extendedLogDataStringEncoding: try binaryValue.field(1),
            events: try binaryValue.field(2)
        )
    }
}

extension AccessLogEventPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            numberOfMediaRequests.binaryValue,
            playbackStartDate.binaryValue,
            uri.binaryValue,
            serverAddress.binaryValue,
            numberOfServerAddressChanges.binaryValue,
            playbackSessionId.binaryValue,
            playbackStartOffset.binaryValue,
            segmentsDownloadedDuration.binaryValue,
            durationWatched.binaryValue,
            numberOfStalls.binaryValue,
            numberOfBytesTransferred.binaryValue,
            transferDuration.binaryValue,
            observedBitrate.binaryValue,
            indicatedBitrate.binaryValue,
            indicatedAverageBitrate.binaryValue,
            averageVideoBitrate.binaryValue,
            averageAudioBitrate.binaryValue,
            numberOfDroppedVideoFrames.binaryValue,
            startupTime.binaryValue,
            downloadOverdue.binaryValue,
            observedBitrateStandardDeviation.binaryValue,
            playbackType.binaryValue,
            mediaRequestsWwan.binaryValue,
            switchBitrate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            numberOfMediaRequests: try binaryValue.field(0),
            playbackStartDate: try binaryValue.field(1),
            uri: try binaryValue.field(2),
            serverAddress: try binaryValue.field(3),
            numberOfServerAddressChanges: try binaryValue.field(4),
            playbackSessionId: try binaryValue.field(5),
            playbackStartOffset: try binaryValue.field(6),
            segmentsDownloadedDuration: try binaryValue.field(7),
            durationWatched: try binaryValue.field(8),
            numberOfStalls: try binaryValue.field(9),
            numberOfBytesTransferred: try binaryValue.field(10),
            transferDuration: try binaryValue.field(11),
            observedBitrate: try binaryValue.field(12),
            indicatedBitrate: try binaryValue.field(13),
            indicatedAverageBitrate: try binaryValue.field(14),
            averageVideoBitrate: try binaryValue.field(15),
            averageAudioBitrate: try binaryValue.field(16),
            numberOfDroppedVideoFrames: try binaryValue.field(17),
            startupTime: try binaryValue.field(18),
            downloadOverdue: try binaryValue.field(19),
            observedBitrateStandardDeviation: try binaryValue.field(20),
            playbackType: try binaryValue.field(21),
            mediaRequestsWwan: try binaryValue.field(22),
            switchBitrate: try binaryValue.field(23)
        )
    }
}

extension ErrorLogPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            extendedLog.binaryValue,
            extendedLogDataStringEncoding.binaryValue,
            events.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            extendedLog: try binaryValue.field(0),
            extendedLogDataStringEncoding: try binaryValue.field(1),
            events: try binaryValue.field(2)
        )
    }
}

extension ErrorLogEventPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            date.binaryValue,
            uri.binaryValue,
            serverAddress.binaryValue,
            playbackSessionId.binaryValue,
            errorStatusCode.binaryValue,
            errorDomain.binaryValue,
            errorComment.binaryValue,
            allHttpResponseHeaderFields.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            date: try binaryValue.field(0),
            uri: try binaryValue.field(1),
            serverAddress: try binaryValue.field(2),
            playbackSessionId: try binaryValue.field(3),
            errorStatusCode: try binaryValue.field(4),
            errorDomain: try binaryValue.field(5),
            errorComment: try binaryValue.field(6),
            allHttpResponseHeaderFields: try binaryValue.field(7)
        )
    }
}

extension PlayerLayerInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            hasPlayer.binaryValue,
            videoGravity.binaryValue,
            readyForDisplay.binaryValue,
            videoRect.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            hasPlayer: try binaryValue.field(0),
            videoGravity: try binaryValue.field(1),
            readyForDisplay: try binaryValue.field(2),
            videoRect: try binaryValue.field(3)
        )
    }
}

extension PlayerLooperInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            status.binaryValue,
            errorMessage.binaryValue,
            loopCount.binaryValue,
            loopingItemCount.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            status: try binaryValue.field(0),
            errorMessage: try binaryValue.field(1),
            loopCount: try binaryValue.field(2),
            loopingItemCount: try binaryValue.field(3)
        )
    }
}

extension MediaSelectionCriteriaPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            preferredLanguages.binaryValue,
            preferredMediaCharacteristics.binaryValue,
            principalMediaCharacteristics.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            preferredLanguages: try binaryValue.field(0),
            preferredMediaCharacteristics: try binaryValue.field(1),
            principalMediaCharacteristics: try binaryValue.field(2)
        )
    }
}

extension VideoOutputInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            suppressesPlayerRendering.binaryValue,
            hasDelegate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            suppressesPlayerRendering: try binaryValue.field(0),
            hasDelegate: try binaryValue.field(1)
        )
    }
}

extension MetadataOutputInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            suppressesPlayerRendering.binaryValue,
            advanceIntervalForDelegateInvocation.binaryValue,
            identifiers.binaryValue,
            hasDelegate.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            suppressesPlayerRendering: try binaryValue.field(0),
            advanceIntervalForDelegateInvocation: try binaryValue.field(1),
            identifiers: try binaryValue.field(2),
            hasDelegate: try binaryValue.field(3)
        )
    }
}

extension LegibleOutputInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            suppressesPlayerRendering.binaryValue,
            advanceIntervalForDelegateInvocation.binaryValue,
            nativeRepresentationSubtypes.binaryValue,
            hasDelegate.binaryValue,
            textStylingResolution.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            suppressesPlayerRendering: try binaryValue.field(0),
            advanceIntervalForDelegateInvocation: try binaryValue.field(1),
            nativeRepresentationSubtypes: try binaryValue.field(2),
            hasDelegate: try binaryValue.field(3),
            textStylingResolution: try binaryValue.field(4)
        )
    }
}
