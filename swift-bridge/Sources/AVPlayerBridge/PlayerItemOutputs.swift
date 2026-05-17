import AVFoundation
import Foundation

class AVPPlayerItemOutputBox: NSObject {
    var output: AVPlayerItemOutput {
        fatalError("override in subclass")
    }
}

final class AVPPlayerItemVideoOutputBox: AVPPlayerItemOutputBox {
    let videoOutput: AVPlayerItemVideoOutput

    init(videoOutput: AVPlayerItemVideoOutput) {
        self.videoOutput = videoOutput
    }

    override var output: AVPlayerItemOutput {
        videoOutput
    }
}

final class AVPPlayerItemMetadataOutputBox: AVPPlayerItemOutputBox {
    let metadataOutput: AVPlayerItemMetadataOutput
    let identifiers: [String]?

    init(metadataOutput: AVPlayerItemMetadataOutput, identifiers: [String]?) {
        self.metadataOutput = metadataOutput
        self.identifiers = identifiers
    }

    override var output: AVPlayerItemOutput {
        metadataOutput
    }
}

final class AVPPlayerItemLegibleOutputBox: AVPPlayerItemOutputBox {
    let legibleOutput: AVPlayerItemLegibleOutput
    let nativeRepresentationSubtypes: [UInt32]

    init(legibleOutput: AVPlayerItemLegibleOutput, nativeRepresentationSubtypes: [UInt32]) {
        self.legibleOutput = legibleOutput
        self.nativeRepresentationSubtypes = nativeRepresentationSubtypes
    }

    override var output: AVPlayerItemOutput {
        legibleOutput
    }
}

@_cdecl("av_player_item_output_release")
public func av_player_item_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    guard let outputPtr else { return }
    Unmanaged<AVPPlayerItemOutputBox>.fromOpaque(outputPtr).release()
}

@_cdecl("av_player_item_output_suppresses_player_rendering")
public func av_player_item_output_suppresses_player_rendering(
    _ outputPtr: UnsafeMutableRawPointer
) -> Bool {
    let output = Unmanaged<AVPPlayerItemOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    return output.suppressesPlayerRendering
}

@_cdecl("av_player_item_output_set_suppresses_player_rendering")
public func av_player_item_output_set_suppresses_player_rendering(
    _ outputPtr: UnsafeMutableRawPointer,
    _ suppresses: Bool
) {
    let output = Unmanaged<AVPPlayerItemOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    output.suppressesPlayerRendering = suppresses
}

@_cdecl("av_player_item_output_item_time_for_host_time_json")
public func av_player_item_output_item_time_for_host_time_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ hostTime: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = Unmanaged<AVPPlayerItemOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    do {
        return ffiString(try avpEncodeJSON(encodeTime(output.itemTime(forHostTime: hostTime))))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_output_item_time_for_mach_absolute_time_json")
public func av_player_item_output_item_time_for_mach_absolute_time_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ machAbsoluteTime: Int64,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = Unmanaged<AVPPlayerItemOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    do {
        return ffiString(try avpEncodeJSON(encodeTime(output.itemTime(forMachAbsoluteTime: machAbsoluteTime))))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_video_output_create")
public func av_player_item_video_output_create(
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let output: AVPlayerItemVideoOutput
        if let settingsJson {
            let settings = try videoOutputSettingsDictionary(avpDecodeJSON(settingsJson, as: VideoOutputSettingsPayload.self))
            output = AVPlayerItemVideoOutput(outputSettings: settings)
        } else {
            output = AVPlayerItemVideoOutput(pixelBufferAttributes: nil)
        }
        let box = AVPPlayerItemVideoOutputBox(videoOutput: output)
        return Unmanaged.passRetained(box).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_video_output_info_json")
public func av_player_item_video_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let box = Unmanaged<AVPPlayerItemVideoOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let payload = VideoOutputInfoPayload(
        suppressesPlayerRendering: box.videoOutput.suppressesPlayerRendering,
        hasDelegate: box.videoOutput.delegate != nil
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_video_output_has_new_pixel_buffer_for_item_time")
public func av_player_item_video_output_has_new_pixel_buffer_for_item_time(
    _ outputPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) -> Bool {
    let box = Unmanaged<AVPPlayerItemVideoOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    return box.videoOutput.hasNewPixelBuffer(forItemTime: cmTime(value: value, timescale: timescale, kind: kind))
}

@_cdecl("av_player_item_video_output_copy_pixel_buffer_for_item_time")
public func av_player_item_video_output_copy_pixel_buffer_for_item_time(
    _ outputPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) -> UnsafeMutableRawPointer? {
    let box = Unmanaged<AVPPlayerItemVideoOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let itemTime = cmTime(value: value, timescale: timescale, kind: kind)
    guard let pixelBuffer = box.videoOutput.copyPixelBuffer(forItemTime: itemTime, itemTimeForDisplay: nil) else {
        return nil
    }
    return Unmanaged.passRetained(pixelBuffer).toOpaque()
}

@_cdecl("av_player_item_metadata_output_create")
public func av_player_item_metadata_output_create(
    _ identifiersJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let identifiers = try identifiersJson.map { try avpDecodeJSON($0, as: [String].self) }
        let output = AVPlayerItemMetadataOutput(identifiers: identifiers)
        let box = AVPPlayerItemMetadataOutputBox(metadataOutput: output, identifiers: identifiers)
        return Unmanaged.passRetained(box).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_metadata_output_info_json")
public func av_player_item_metadata_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let box = Unmanaged<AVPPlayerItemMetadataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let payload = MetadataOutputInfoPayload(
        suppressesPlayerRendering: box.metadataOutput.suppressesPlayerRendering,
        advanceIntervalForDelegateInvocation: box.metadataOutput.advanceIntervalForDelegateInvocation,
        identifiers: box.identifiers,
        hasDelegate: box.metadataOutput.delegate != nil
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_metadata_output_set_advance_interval")
public func av_player_item_metadata_output_set_advance_interval(
    _ outputPtr: UnsafeMutableRawPointer,
    _ interval: Double
) {
    let box = Unmanaged<AVPPlayerItemMetadataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    box.metadataOutput.advanceIntervalForDelegateInvocation = interval
}

@_cdecl("av_player_item_legible_output_create")
public func av_player_item_legible_output_create(
    _ nativeRepresentationSubtypesJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let nativeRepresentationSubtypes = try nativeRepresentationSubtypesJson.map {
            try avpDecodeJSON($0, as: [UInt32].self)
        } ?? []
        let output: AVPlayerItemLegibleOutput
        if nativeRepresentationSubtypes.isEmpty {
            output = AVPlayerItemLegibleOutput()
        } else {
            output = AVPlayerItemLegibleOutput(
                mediaSubtypesForNativeRepresentation: nativeRepresentationSubtypes.map(NSNumber.init(value:))
            )
        }
        let box = AVPPlayerItemLegibleOutputBox(
            legibleOutput: output,
            nativeRepresentationSubtypes: nativeRepresentationSubtypes
        )
        return Unmanaged.passRetained(box).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_legible_output_info_json")
public func av_player_item_legible_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let box = Unmanaged<AVPPlayerItemLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let payload = LegibleOutputInfoPayload(
        suppressesPlayerRendering: box.legibleOutput.suppressesPlayerRendering,
        advanceIntervalForDelegateInvocation: box.legibleOutput.advanceIntervalForDelegateInvocation,
        nativeRepresentationSubtypes: box.nativeRepresentationSubtypes,
        hasDelegate: box.legibleOutput.delegate != nil,
        textStylingResolution: avpLegibleOutputTextStylingResolutionString(box.legibleOutput.textStylingResolution)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_legible_output_set_advance_interval")
public func av_player_item_legible_output_set_advance_interval(
    _ outputPtr: UnsafeMutableRawPointer,
    _ interval: Double
) {
    let box = Unmanaged<AVPPlayerItemLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    box.legibleOutput.advanceIntervalForDelegateInvocation = interval
}

@_cdecl("av_player_item_legible_output_set_text_styling_resolution")
public func av_player_item_legible_output_set_text_styling_resolution(
    _ outputPtr: UnsafeMutableRawPointer,
    _ resolutionPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let box = Unmanaged<AVPPlayerItemLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let rawValue = String(cString: resolutionPtr)
    guard let resolution = avpLegibleOutputTextStylingResolution(from: rawValue) else {
        outErrorMessage?.pointee = ffiString("invalid AVPlayerItemLegibleOutputTextStylingResolution: \(rawValue)")
        return AVP_INVALID_ARGUMENT
    }
    box.legibleOutput.textStylingResolution = resolution
    return AVP_OK
}

private func avpLegibleOutputTextStylingResolutionString(
    _ resolution: AVPlayerItemLegibleOutput.TextStylingResolution
) -> String {
    if resolution == .default {
        return "default"
    }
    if resolution == .sourceAndRulesOnly {
        return "source_and_rules_only"
    }
    return resolution.rawValue
}

private func avpLegibleOutputTextStylingResolution(
    from rawValue: String
) -> AVPlayerItemLegibleOutput.TextStylingResolution? {
    if rawValue == "default" || rawValue == AVPlayerItemLegibleOutput.TextStylingResolution.default.rawValue {
        return .default
    }
    if rawValue == "source_and_rules_only"
        || rawValue == AVPlayerItemLegibleOutput.TextStylingResolution.sourceAndRulesOnly.rawValue {
        return .sourceAndRulesOnly
    }
    return nil
}

private func videoOutputSettingsDictionary(_ payload: VideoOutputSettingsPayload) throws -> [String: Any] {
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
