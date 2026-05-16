import AVFoundation
import CoreMedia
import CoreVideo
import Foundation

private let linearPcmFormatId: UInt32 = 0x6c70636d

@_cdecl("av_reader_create")
public func av_reader_create(
    _ assetPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    do {
        let reader = try AVAssetReader(asset: asset)
        return Unmanaged.passRetained(reader).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_reader_release")
public func av_reader_release(_ readerPtr: UnsafeMutableRawPointer?) {
    guard let readerPtr else { return }
    Unmanaged<AVAssetReader>.fromOpaque(readerPtr).release()
}

@_cdecl("av_reader_info_json")
public func av_reader_info_json(
    _ readerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let reader = Unmanaged<AVAssetReader>.fromOpaque(readerPtr).takeUnretainedValue()
    let payload = ReaderInfoPayload(
        status: Int32(reader.status.rawValue),
        errorMessage: reader.error?.localizedDescription,
        timeRange: encodeTimeRange(reader.timeRange),
        outputCount: reader.outputs.count
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_reader_set_time_range")
public func av_reader_set_time_range(
    _ readerPtr: UnsafeMutableRawPointer,
    _ startValue: Int64,
    _ startTimescale: Int32,
    _ startKind: Int32,
    _ durationValue: Int64,
    _ durationTimescale: Int32,
    _ durationKind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let reader = Unmanaged<AVAssetReader>.fromOpaque(readerPtr).takeUnretainedValue()
    guard reader.status == .unknown else {
        outErrorMessage?.pointee = ffiString("timeRange can only be set before startReading")
        return AVP_OPERATION_FAILED
    }
    reader.timeRange = CMTimeRange(
        start: cmTime(value: startValue, timescale: startTimescale, kind: startKind),
        duration: cmTime(value: durationValue, timescale: durationTimescale, kind: durationKind)
    )
    return AVP_OK
}

@_cdecl("av_reader_start")
public func av_reader_start(
    _ readerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let reader = Unmanaged<AVAssetReader>.fromOpaque(readerPtr).takeUnretainedValue()
    if reader.startReading() {
        return AVP_OK
    }
    outErrorMessage?.pointee = ffiString(reader.error?.localizedDescription ?? "startReading returned false")
    return AVP_OPERATION_FAILED
}

@_cdecl("av_reader_cancel")
public func av_reader_cancel(_ readerPtr: UnsafeMutableRawPointer) {
    let reader = Unmanaged<AVAssetReader>.fromOpaque(readerPtr).takeUnretainedValue()
    reader.cancelReading()
}

@_cdecl("av_reader_can_add_output")
public func av_reader_can_add_output(
    _ readerPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) -> Bool {
    let reader = Unmanaged<AVAssetReader>.fromOpaque(readerPtr).takeUnretainedValue()
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    return reader.canAdd(output)
}

@_cdecl("av_reader_add_output")
public func av_reader_add_output(
    _ readerPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let reader = Unmanaged<AVAssetReader>.fromOpaque(readerPtr).takeUnretainedValue()
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    guard reader.status == .unknown else {
        outErrorMessage?.pointee = ffiString("outputs can only be added before startReading")
        return AVP_OPERATION_FAILED
    }
    guard reader.canAdd(output) else {
        outErrorMessage?.pointee = ffiString("reader cannot add output")
        return AVP_OPERATION_FAILED
    }
    reader.add(output)
    return AVP_OK
}

@_cdecl("av_reader_track_output_create_video")
public func av_reader_track_output_create_video(
    _ trackPtr: UnsafeMutableRawPointer,
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let track = Unmanaged<AVAssetTrack>.fromOpaque(trackPtr).takeUnretainedValue()
        let settings = try settingsJson.map {
            try videoSettingsDictionary(avpDecodeJSON($0, as: VideoOutputSettingsPayload.self))
        }
        let output = AVAssetReaderTrackOutput(track: track, outputSettings: settings ?? nil)
        return Unmanaged.passRetained(output).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_reader_track_output_create_audio")
public func av_reader_track_output_create_audio(
    _ trackPtr: UnsafeMutableRawPointer,
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let track = Unmanaged<AVAssetTrack>.fromOpaque(trackPtr).takeUnretainedValue()
        let settings = try settingsJson.map {
            try audioSettingsDictionary(avpDecodeJSON($0, as: AudioOutputSettingsPayload.self))
        }
        let output = AVAssetReaderTrackOutput(track: track, outputSettings: settings ?? nil)
        return Unmanaged.passRetained(output).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_reader_track_output_create_passthrough")
public func av_reader_track_output_create_passthrough(
    _ trackPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let track = Unmanaged<AVAssetTrack>.fromOpaque(trackPtr).takeUnretainedValue()
    let output = AVAssetReaderTrackOutput(track: track, outputSettings: nil)
    return Unmanaged.passRetained(output).toOpaque()
}

@_cdecl("av_reader_audio_mix_output_create")
public func av_reader_audio_mix_output_create(
    _ trackPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int,
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let tracks = try tracksFromPointers(trackPtrs, count: count)
        let settings = try settingsJson.map {
            try audioSettingsDictionary(avpDecodeJSON($0, as: AudioOutputSettingsPayload.self))
        }
        let output = AVAssetReaderAudioMixOutput(audioTracks: tracks, audioSettings: settings ?? nil)
        return Unmanaged.passRetained(output).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_reader_video_composition_output_create")
public func av_reader_video_composition_output_create(
    _ trackPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int,
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let tracks = try tracksFromPointers(trackPtrs, count: count)
        let settings = try settingsJson.map {
            try videoSettingsDictionary(avpDecodeJSON($0, as: VideoOutputSettingsPayload.self))
        }
        let output = AVAssetReaderVideoCompositionOutput(videoTracks: tracks, videoSettings: settings ?? nil)
        return Unmanaged.passRetained(output).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_reader_output_release")
public func av_reader_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    guard let outputPtr else { return }
    Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).release()
}

@_cdecl("av_reader_output_set_always_copies_sample_data")
public func av_reader_output_set_always_copies_sample_data(
    _ outputPtr: UnsafeMutableRawPointer,
    _ alwaysCopies: Bool
) {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    output.alwaysCopiesSampleData = alwaysCopies
}

@_cdecl("av_reader_output_media_type")
public func av_reader_output_media_type(
    _ outputPtr: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    return ffiString(avpMediaTypeString(output.mediaType))
}

@_cdecl("av_reader_output_copy_next_sample_buffer")
public func av_reader_output_copy_next_sample_buffer(
    _ outputPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    guard let sample = output.copyNextSampleBuffer() else { return nil }
    return Unmanaged.passRetained(sample).toOpaque()
}

@_cdecl("av_reader_output_copy_next_video_pixel_buffer")
public func av_reader_output_copy_next_video_pixel_buffer(
    _ outputPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let output = Unmanaged<AVAssetReaderOutput>.fromOpaque(outputPtr).takeUnretainedValue()
    guard let sample = output.copyNextSampleBuffer() else { return nil }
    guard let pixelBuffer = CMSampleBufferGetImageBuffer(sample) else { return nil }
    return Unmanaged.passRetained(pixelBuffer).toOpaque()
}

private func tracksFromPointers(
    _ trackPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    count: Int
) throws -> [AVAssetTrack] {
    guard let trackPtrs else {
        throw BridgeError.message("missing track pointer array")
    }
    return (0..<count).map { index in
        Unmanaged<AVAssetTrack>.fromOpaque(trackPtrs[index]!).takeUnretainedValue()
    }
}

private func videoSettingsDictionary(_ payload: VideoOutputSettingsPayload) throws -> [String: Any] {
    var settings: [String: Any] = [
        kCVPixelBufferPixelFormatTypeKey as String: Int(payload.pixelFormat)
    ]
    if let width = payload.width {
        settings[kCVPixelBufferWidthKey as String] = width
    }
    if let height = payload.height {
        settings[kCVPixelBufferHeightKey as String] = height
    }
    return settings
}

private func audioSettingsDictionary(_ payload: AudioOutputSettingsPayload) throws -> [String: Any] {
    var settings: [String: Any] = [
        AVFormatIDKey: linearPcmFormatId,
        AVLinearPCMBitDepthKey: Int(payload.bitsPerChannel),
        AVLinearPCMIsFloatKey: payload.isFloat,
        AVLinearPCMIsBigEndianKey: false,
        AVLinearPCMIsNonInterleaved: payload.isNonInterleaved,
    ]
    if let sampleRate = payload.sampleRate {
        settings[AVSampleRateKey] = sampleRate
    }
    if let channelCount = payload.channelCount {
        settings[AVNumberOfChannelsKey] = Int(channelCount)
    }
    return settings
}
