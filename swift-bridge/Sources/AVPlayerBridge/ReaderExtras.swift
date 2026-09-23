import AVFoundation
import AVPlayerObjCBridge
import CoreMedia
import Foundation

private struct AVPCaptionGroupPayload: Codable {
    let timeRange: TimeRangePayload
    let captions: [String]
}

private struct AVPCaptionValidationEventPayload: Codable {
    let captionText: String
    let syntaxElements: [String]
}

final class CaptionValidationObserverBox: NSObject, AVAssetReaderCaptionValidationHandling {
    private weak var adaptor: AVAssetReaderOutputCaptionAdaptor?
    private let callback: AVPJsonCallback
    private let gate: AVPCallbackGate

    init(
        adaptor: AVAssetReaderOutputCaptionAdaptor,
        callback: @escaping AVPJsonCallback,
        gate: AVPCallbackGate
    ) {
        self.adaptor = adaptor
        self.callback = callback
        self.gate = gate
        super.init()
        adaptor.validationDelegate = self
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        adaptor?.validationDelegate = nil
        gate.finish()
    }

    func captionAdaptor(
        _ adaptor: AVAssetReaderOutputCaptionAdaptor,
        didVendCaption caption: AVCaption,
        skippingUnsupportedSourceSyntaxElements syntaxElements: [String]
    ) {
        gate.deliverJSON(
            AVPCaptionValidationEventPayload(
                captionText: caption.text,
                syntaxElements: syntaxElements
            ),
            to: callback
        )
    }
}

@_cdecl("av_reader_output_supports_random_access")
public func av_reader_output_supports_random_access(_ outputPtr: UnsafeMutableRawPointer) -> Bool {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    if #available(macOS 10.10, *) {
        return output.supportsRandomAccess
    }
    return false
}

@_cdecl("av_reader_output_set_supports_random_access")
public func av_reader_output_set_supports_random_access(
    _ outputPtr: UnsafeMutableRawPointer,
    _ supportsRandomAccess: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    var reason: NSString?
    guard AVPTrySetSupportsRandomAccess(output, supportsRandomAccess, &reason) else {
        outErrorMessage?.pointee = ffiString((reason as String?) ?? "supportsRandomAccess cannot be changed")
        return AVP_OPERATION_FAILED
    }
    return AVP_OK
}

func avpReadingTimeRangesError(_ ranges: [CMTimeRange]) -> String? {
    var previousStart: CMTime?
    var previousEnd: CMTime?
    for (index, range) in ranges.enumerated() {
        guard range.start.isNumeric else {
            return "time range \(index) must have a numeric start"
        }
        let duration = range.duration
        let durationIsValid = (duration.isNumeric && duration >= .zero) || duration.isPositiveInfinity
        guard durationIsValid else {
            return "time range \(index) must have a non-negative numeric or positive-infinity duration"
        }
        if let previousStart, range.start <= previousStart {
            return "time range starts must be strictly increasing"
        }
        if let previousEnd, range.start < previousEnd {
            return "time ranges must not overlap"
        }
        previousStart = range.start
        previousEnd = duration.isPositiveInfinity ? .positiveInfinity : CMTimeAdd(range.start, duration)
    }
    return nil
}

@_cdecl("av_reader_output_reset_for_time_ranges_json")
public func av_reader_output_reset_for_time_ranges_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ timeRangesJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 10.10, *) else {
        outErrorMessage?.pointee = ffiString("reader random access requires macOS 10.10")
        return AVP_OPERATION_FAILED
    }
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    do {
        let payloads = try avpDecodeJSON(timeRangesJson, as: [TimeRangePayload].self)
        let ranges = payloads.map(cmTimeRange(from:))
        if let message = avpReadingTimeRangesError(ranges) {
            outErrorMessage?.pointee = ffiString(message)
            return AVP_INVALID_ARGUMENT
        }
        guard output.supportsRandomAccess else {
            outErrorMessage?.pointee = ffiString("resetForReadingTimeRanges requires supportsRandomAccess")
            return AVP_INVALID_ARGUMENT
        }
        var reason: NSString?
        guard AVPTryResetForReadingTimeRanges(output, ranges.map { NSValue(timeRange: $0) }, &reason) else {
            outErrorMessage?.pointee = ffiString(
                (reason as String?) ?? "resetForReadingTimeRanges is not allowed in the current reader state"
            )
            return AVP_OPERATION_FAILED
        }
        return AVP_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVP_OPERATION_FAILED
    }
}

@_cdecl("av_reader_output_mark_configuration_as_final")
public func av_reader_output_mark_configuration_as_final(_ outputPtr: UnsafeMutableRawPointer) {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    if #available(macOS 10.10, *) {
        output.markConfigurationAsFinal()
    }
}

@_cdecl("av_reader_sample_reference_output_create")
public func av_reader_sample_reference_output_create(
    _ trackPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let track = Unmanaged<AVAssetTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    if #available(macOS 10.10, *) {
        return avpRetained(AVAssetReaderSampleReferenceOutput(track: track))
    }
    outErrorMessage?.pointee = ffiString("sample-reference outputs require macOS 10.10")
    return nil
}

@_cdecl("av_reader_sample_reference_output_copy_track")
public func av_reader_sample_reference_output_copy_track(
    _ outputPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.10, *) else { return nil }
    let output = Unmanaged<AVAssetReaderSampleReferenceOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    return avpRetained(output.track)
}

@_cdecl("av_reader_output_metadata_adaptor_create")
public func av_reader_output_metadata_adaptor_create(
    _ trackOutputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.10, *) else {
        outErrorMessage?.pointee = ffiString("metadata adaptors require macOS 10.10")
        return nil
    }
    let trackOutput = Unmanaged<AVAssetReaderTrackOutput>.fromOpaque(trackOutputPtr).takeUnretainedValue()
    var reason: NSString?
    guard let adaptor = AVPTryCreateMetadataAdaptor(trackOutput, &reason) else {
        outErrorMessage?.pointee = ffiString((reason as String?) ?? "AVAssetReaderOutputMetadataAdaptor could not be created")
        return nil
    }
    return avpRetained(adaptor)
}

@_cdecl("av_reader_output_metadata_adaptor_copy_track_output")
public func av_reader_output_metadata_adaptor_copy_track_output(
    _ adaptorPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.10, *) else { return nil }
    let adaptor = Unmanaged<AVAssetReaderOutputMetadataAdaptor>.fromOpaque(adaptorPtr).takeUnretainedValue()
    return avpRetained(adaptor.assetReaderTrackOutput)
}

@_cdecl("av_reader_output_metadata_adaptor_copy_next_timed_metadata_group")
public func av_reader_output_metadata_adaptor_copy_next_timed_metadata_group(
    _ adaptorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.10, *) else {
        outErrorMessage?.pointee = ffiString("metadata adaptors require macOS 10.10")
        return nil
    }
    let adaptor = Unmanaged<AVAssetReaderOutputMetadataAdaptor>.fromOpaque(adaptorPtr).takeUnretainedValue()
    var reason: NSString?
    guard let group = AVPTryNextTimedMetadataGroup(adaptor, &reason) else {
        if let reason {
            outErrorMessage?.pointee = ffiString(reason as String)
        }
        return nil
    }
    return avpRetained(group)
}

@_cdecl("av_reader_output_caption_adaptor_create")
public func av_reader_output_caption_adaptor_create(
    _ trackOutputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("caption adaptors require macOS 12.0")
        return nil
    }
    let trackOutput = Unmanaged<AVAssetReaderTrackOutput>.fromOpaque(trackOutputPtr).takeUnretainedValue()
    var reason: NSString?
    guard let adaptor = AVPTryCreateCaptionAdaptor(trackOutput, &reason) else {
        outErrorMessage?.pointee = ffiString((reason as String?) ?? "AVAssetReaderOutputCaptionAdaptor could not be created")
        return nil
    }
    return avpRetained(adaptor)
}

@_cdecl("av_reader_output_caption_adaptor_copy_track_output")
public func av_reader_output_caption_adaptor_copy_track_output(
    _ adaptorPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    let adaptor = Unmanaged<AVAssetReaderOutputCaptionAdaptor>.fromOpaque(adaptorPtr).takeUnretainedValue()
    return avpRetained(adaptor.assetReaderTrackOutput)
}

@_cdecl("av_reader_output_caption_adaptor_next_caption_group_json")
public func av_reader_output_caption_adaptor_next_caption_group_json(
    _ adaptorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("caption adaptors require macOS 12.0")
        return nil
    }
    let adaptor = Unmanaged<AVAssetReaderOutputCaptionAdaptor>.fromOpaque(adaptorPtr).takeUnretainedValue()
    var reason: NSString?
    guard let group = AVPTryNextCaptionGroup(adaptor, &reason) else {
        if let reason {
            outErrorMessage?.pointee = ffiString(reason as String)
        }
        return nil
    }
    let payload = AVPCaptionGroupPayload(
        timeRange: encodeTimeRange(group.timeRange),
        captions: group.captions.map(\.text)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_reader_output_caption_adaptor_add_validation_observer")
public func av_reader_output_caption_adaptor_add_validation_observer(
    _ adaptorPtr: UnsafeMutableRawPointer,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("caption validation requires macOS 12.0")
        return nil
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing caption validation callback")
        return nil
    }
    let adaptor = Unmanaged<AVAssetReaderOutputCaptionAdaptor>.fromOpaque(adaptorPtr).takeUnretainedValue()
    return avpRetained(
        CaptionValidationObserverBox(
            adaptor: adaptor,
            callback: callback,
            gate: gate
        )
    )
}

@_cdecl("av_reader_output_caption_validation_observer_release")
public func av_reader_output_caption_validation_observer_release(
    _ observerPtr: UnsafeMutableRawPointer?
) {
    guard let observerPtr else { return }
    let observer = Unmanaged<CaptionValidationObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}
