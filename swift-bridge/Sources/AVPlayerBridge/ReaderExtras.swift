import AVFoundation
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
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        adaptor: AVAssetReaderOutputCaptionAdaptor,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.adaptor = adaptor
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        adaptor.validationDelegate = self
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        adaptor?.validationDelegate = nil
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    func captionAdaptor(
        _ adaptor: AVAssetReaderOutputCaptionAdaptor,
        didVendCaption caption: AVCaption,
        skippingUnsupportedSourceSyntaxElements syntaxElements: [String]
    ) {
        guard !disposed else { return }
        let payload = AVPCaptionValidationEventPayload(
            captionText: caption.text,
            syntaxElements: syntaxElements
        )
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
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
    _ supportsRandomAccess: Bool
) {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    if #available(macOS 10.10, *) {
        output.supportsRandomAccess = supportsRandomAccess
    }
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
        output.reset(forReadingTimeRanges: payloads.map { NSValue(timeRange: cmTimeRange(from: $0)) })
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
    return avpRetained(AVAssetReaderOutputMetadataAdaptor(assetReaderTrackOutput: trackOutput))
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
    guard let group = adaptor.nextTimedMetadataGroup() else { return nil }
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
    return avpRetained(AVAssetReaderOutputCaptionAdaptor(assetReaderTrackOutput: trackOutput))
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
    guard let group = adaptor.nextCaptionGroup() else { return nil }
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
            userData: userData,
            dropUserData: dropUserData
        )
    )
}

@_cdecl("av_reader_output_caption_validation_observer_release")
public func av_reader_output_caption_validation_observer_release(
    _ observerPtr: UnsafeMutableRawPointer?
) {
    guard let observerPtr else { return }
    Unmanaged<CaptionValidationObserverBox>.fromOpaque(observerPtr).release()
}
