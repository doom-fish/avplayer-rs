import AVPlayerABI
import AVFoundation
import CoreMedia
import CoreVideo
import Foundation

struct PlayerVideoOutputTagCollectionPayload: AVPBridge {
    let tags: [String]
}

struct AffineTransformPayload: AVPBridge {
    let a: Double
    let b: Double
    let c: Double
    let d: Double
    let tx: Double
    let ty: Double
}

struct PlayerVideoOutputConfigurationPayload: AVPBridge {
    let hasSourcePlayerItem: Bool
    let dataChannelDescriptions: [[String]]
    let preferredTransform: AffineTransformPayload?
    let activationTime: TimePayload
}

struct PlayerVideoTaggedBufferPayload: AVPBridge {
    let tags: [String]
    let bufferKind: String
    let pixelBufferWidth: Int?
    let pixelBufferHeight: Int?
}

struct PlayerVideoOutputSamplePayload: AVPBridge {
    let taggedBuffers: [PlayerVideoTaggedBufferPayload]
    let presentationTime: TimePayload
    let activeConfiguration: PlayerVideoOutputConfigurationPayload
}

@available(macOS 14.0, *)
final class AVPPlayerVideoOutputTagCollectionBox: NSObject {
    let tags: [CMTag]

    init(tags: [CMTag]) {
        self.tags = tags
    }
}

@available(macOS 14.2, *)
final class AVPVideoOutputSpecificationBox: NSObject {
    let specification: AVVideoOutputSpecification

    init(specification: AVVideoOutputSpecification) {
        self.specification = specification
    }
}

@available(macOS 14.2, *)
final class AVPPlayerVideoOutputBox: NSObject {
    let videoOutput: AVPlayerVideoOutput

    init(videoOutput: AVPlayerVideoOutput) {
        self.videoOutput = videoOutput
    }
}

@_cdecl("av_player_video_output_tag_collection_create_with_preset")
public func av_player_video_output_tag_collection_create_with_preset(
    _ presetRaw: UInt32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.2, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerVideoOutput tag collections require macOS 14.2+")
        return nil
    }

    let tags: [CMTag]
    switch presetRaw {
    case 0:
        tags = [CMTag.stereoView([])]
    case 1:
        tags = [CMTag.stereoView([.leftEye, .rightEye])]
    default:
        outErrorMessage?.pointee = ffiString("invalid CMTagCollectionVideoOutputPreset raw value: \(presetRaw)")
        return nil
    }

    return Unmanaged.passRetained(AVPPlayerVideoOutputTagCollectionBox(tags: tags)).toOpaque()
}

@_cdecl("av_player_video_output_tag_collection_release")
public func av_player_video_output_tag_collection_release(_ tagCollectionPtr: UnsafeMutableRawPointer?) {
    guard let tagCollectionPtr else { return }
    if #available(macOS 14.0, *) {
        Unmanaged<AVPPlayerVideoOutputTagCollectionBox>.fromOpaque(tagCollectionPtr).release()
    }
}

@_cdecl("av_player_video_output_tag_collection_info_value")
public func av_player_video_output_tag_collection_info_value(
    _ tagCollectionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerVideoOutput tag collections require macOS 14.0+")
        return nil
    }

    let box = Unmanaged<AVPPlayerVideoOutputTagCollectionBox>.fromOpaque(tagCollectionPtr).takeUnretainedValue()
    let payload = PlayerVideoOutputTagCollectionPayload(tags: box.tags.map(\.description))
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_video_output_specification_create")
public func av_video_output_specification_create(
    _ tagCollectionPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.2, *) else {
        outErrorMessage?.pointee = ffiString("AVVideoOutputSpecification requires macOS 14.2+")
        return nil
    }
    guard let tagCollectionPtrs, count > 0 else {
        outErrorMessage?.pointee = ffiString("video output specifications require at least one tag collection")
        return nil
    }

    let tagCollections = (0..<count).compactMap { index -> [CMTag]? in
        guard let ptr = tagCollectionPtrs[index] else { return nil }
        let box = Unmanaged<AVPPlayerVideoOutputTagCollectionBox>.fromOpaque(ptr).takeUnretainedValue()
        return box.tags
    }
    guard tagCollections.count == count else {
        outErrorMessage?.pointee = ffiString("video output specification tag collection list contained nil")
        return nil
    }

    let specification = AVVideoOutputSpecification(tagCollections: tagCollections)
    return Unmanaged.passRetained(AVPVideoOutputSpecificationBox(specification: specification)).toOpaque()
}

@_cdecl("av_video_output_specification_release")
public func av_video_output_specification_release(_ specificationPtr: UnsafeMutableRawPointer?) {
    guard let specificationPtr else { return }
    if #available(macOS 14.2, *) {
        Unmanaged<AVPVideoOutputSpecificationBox>.fromOpaque(specificationPtr).release()
    }
}

@_cdecl("av_video_output_specification_info_value")
public func av_video_output_specification_info_value(
    _ specificationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 14.2, *) else {
        outErrorMessage?.pointee = ffiString("AVVideoOutputSpecification requires macOS 14.2+")
        return nil
    }

    let box = Unmanaged<AVPVideoOutputSpecificationBox>.fromOpaque(specificationPtr).takeUnretainedValue()
    let payload = box.specification.preferredTagCollections.map {
        PlayerVideoOutputTagCollectionPayload(tags: $0.map(\.description))
    }
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_video_output_specification_set_default_output_settings")
public func av_video_output_specification_set_default_output_settings(
    _ specificationPtr: UnsafeMutableRawPointer,
    _ settingsValue: UnsafePointer<AVPValue>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    guard #available(macOS 14.2, *) else {
        outErrorMessage?.pointee = ffiString("AVVideoOutputSpecification requires macOS 14.2+")
        return AVP_OPERATION_FAILED
    }
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("defaultOutputSettings requires macOS 15.0+")
        return AVP_OPERATION_FAILED
    }

    let box = Unmanaged<AVPVideoOutputSpecificationBox>.fromOpaque(specificationPtr).takeUnretainedValue()
    do {
        let settings = try settingsValue.map {
            try videoOutputSettingsDictionary(avpDecodeValue($0, as: VideoOutputSettingsPayload.self))
        }
        box.specification.defaultOutputSettings = settings
        return AVP_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVP_OPERATION_FAILED
    }
}

@_cdecl("av_video_output_specification_set_output_settings_for_tag_collection")
public func av_video_output_specification_set_output_settings_for_tag_collection(
    _ specificationPtr: UnsafeMutableRawPointer,
    _ settingsValue: UnsafePointer<AVPValue>?,
    _ tagCollectionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    guard #available(macOS 14.2, *) else {
        outErrorMessage?.pointee = ffiString("AVVideoOutputSpecification requires macOS 14.2+")
        return AVP_OPERATION_FAILED
    }
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("setOutputSettings(_:for:) requires macOS 15.0+")
        return AVP_OPERATION_FAILED
    }

    let box = Unmanaged<AVPVideoOutputSpecificationBox>.fromOpaque(specificationPtr).takeUnretainedValue()
    let tagCollectionBox = Unmanaged<AVPPlayerVideoOutputTagCollectionBox>.fromOpaque(tagCollectionPtr).takeUnretainedValue()
    do {
        let settings = try settingsValue.map {
            try videoOutputSettingsDictionary(avpDecodeValue($0, as: VideoOutputSettingsPayload.self))
        }
        box.specification.setOutputSettings(settings, for: tagCollectionBox.tags)
        return AVP_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVP_OPERATION_FAILED
    }
}

@_cdecl("av_player_video_output_create")
public func av_player_video_output_create(
    _ specificationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.2, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerVideoOutput requires macOS 14.2+")
        return nil
    }

    let specification = Unmanaged<AVPVideoOutputSpecificationBox>.fromOpaque(specificationPtr).takeUnretainedValue().specification
    let output = AVPlayerVideoOutput(specification: specification)
    return Unmanaged.passRetained(AVPPlayerVideoOutputBox(videoOutput: output)).toOpaque()
}

@_cdecl("av_player_video_output_release")
public func av_player_video_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    guard let outputPtr else { return }
    if #available(macOS 14.2, *) {
        Unmanaged<AVPPlayerVideoOutputBox>.fromOpaque(outputPtr).release()
    }
}

@_cdecl("av_player_video_output_sample_value")
public func av_player_video_output_sample_value(
    _ outputPtr: UnsafeMutableRawPointer,
    _ hostTimeValue: Int64,
    _ hostTimeTimescale: Int32,
    _ hostTimeKind: Int32,
    _ outBuffers: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 14.2, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerVideoOutput requires macOS 14.2+")
        return nil
    }

    let output = Unmanaged<AVPPlayerVideoOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().videoOutput
    let hostTime = cmTime(value: hostTimeValue, timescale: hostTimeTimescale, kind: hostTimeKind)
    let taggedSample = output.taggedBuffers(forHostTime: hostTime)
    let sample = taggedSample.map { sample in
        PlayerVideoOutputSamplePayload(
            taggedBuffers: sample.taggedBufferGroup.map(encodeTaggedBuffer),
            presentationTime: encodeTime(sample.presentationTime),
            activeConfiguration: encodePlayerVideoOutputConfiguration(sample.activeConfiguration)
        )
    }
    do {
        let value = try avpEncodeValue(sample)
        if let taggedSample {
            let buffers = taggedSample.taggedBufferGroup.map(avpTaggedBufferObject)
            outBuffers?.pointee = Unmanaged.passRetained(AVPObjectArrayBox(objects: buffers)).toOpaque()
        }
        return value
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@available(macOS 14.0, *)
private func avpTaggedBufferObject(_ taggedBuffer: CMTaggedBuffer) -> AnyObject {
    switch taggedBuffer.buffer {
    case .pixelBuffer(let pixelBuffer):
        return pixelBuffer
    case .sampleBuffer(let sampleBuffer):
        return sampleBuffer
    @unknown default:
        return NSNull()
    }
}

@_cdecl("av_player_set_video_output")
public func av_player_set_video_output(
    _ playerPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer?
) {
    guard #available(macOS 14.2, *) else { return }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let output = outputPtr.map { Unmanaged<AVPPlayerVideoOutputBox>.fromOpaque($0).takeUnretainedValue().videoOutput }
    player.videoOutput = output
}

@_cdecl("av_player_copy_video_output")
public func av_player_copy_video_output(_ playerPtr: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.2, *) else { return nil }
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    guard let output = player.videoOutput else { return nil }
    return Unmanaged.passRetained(AVPPlayerVideoOutputBox(videoOutput: output)).toOpaque()
}

@available(macOS 14.2, *)
private func encodePlayerVideoOutputConfiguration(
    _ configuration: AVPlayerVideoOutput.Configuration
) -> PlayerVideoOutputConfigurationPayload {
    PlayerVideoOutputConfigurationPayload(
        hasSourcePlayerItem: configuration.sourcePlayerItem != nil,
        dataChannelDescriptions: configuration.dataChannelDescription.map { $0.map(\.description) },
        preferredTransform: {
            if #available(macOS 15.0, *) {
                return encodeAffineTransform(configuration.preferredTransform)
            }
            return nil
        }(),
        activationTime: encodeTime(configuration.activationTime)
    )
}

private func encodeAffineTransform(_ transform: CGAffineTransform) -> AffineTransformPayload {
    AffineTransformPayload(
        a: transform.a,
        b: transform.b,
        c: transform.c,
        d: transform.d,
        tx: transform.tx,
        ty: transform.ty
    )
}

@available(macOS 14.0, *)
private func encodeTaggedBuffer(_ taggedBuffer: CMTaggedBuffer) -> PlayerVideoTaggedBufferPayload {
    let pixelBufferSize: (Int?, Int?)
    let bufferKind: String
    switch taggedBuffer.buffer {
    case .pixelBuffer(let pixelBuffer):
        bufferKind = "pixel_buffer"
        pixelBufferSize = (CVPixelBufferGetWidth(pixelBuffer), CVPixelBufferGetHeight(pixelBuffer))
    case .sampleBuffer:
        bufferKind = "sample_buffer"
        pixelBufferSize = (nil, nil)
    @unknown default:
        bufferKind = "unknown"
        pixelBufferSize = (nil, nil)
    }
    return PlayerVideoTaggedBufferPayload(
        tags: taggedBuffer.tags.map(\.description),
        bufferKind: bufferKind,
        pixelBufferWidth: pixelBufferSize.0,
        pixelBufferHeight: pixelBufferSize.1
    )
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

extension PlayerVideoOutputTagCollectionPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            tags.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            tags: try binaryValue.field(0)
        )
    }
}

extension AffineTransformPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            a.binaryValue,
            b.binaryValue,
            c.binaryValue,
            d.binaryValue,
            tx.binaryValue,
            ty.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            a: try binaryValue.field(0),
            b: try binaryValue.field(1),
            c: try binaryValue.field(2),
            d: try binaryValue.field(3),
            tx: try binaryValue.field(4),
            ty: try binaryValue.field(5)
        )
    }
}

extension PlayerVideoOutputConfigurationPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            hasSourcePlayerItem.binaryValue,
            dataChannelDescriptions.binaryValue,
            preferredTransform.binaryValue,
            activationTime.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            hasSourcePlayerItem: try binaryValue.field(0),
            dataChannelDescriptions: try binaryValue.field(1),
            preferredTransform: try binaryValue.field(2),
            activationTime: try binaryValue.field(3)
        )
    }
}

extension PlayerVideoTaggedBufferPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            tags.binaryValue,
            bufferKind.binaryValue,
            pixelBufferWidth.binaryValue,
            pixelBufferHeight.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            tags: try binaryValue.field(0),
            bufferKind: try binaryValue.field(1),
            pixelBufferWidth: try binaryValue.field(2),
            pixelBufferHeight: try binaryValue.field(3)
        )
    }
}

extension PlayerVideoOutputSamplePayload {
    var binaryValue: AVPBinaryValue {
        .array([
            taggedBuffers.binaryValue,
            presentationTime.binaryValue,
            activeConfiguration.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            taggedBuffers: try binaryValue.field(0),
            presentationTime: try binaryValue.field(1),
            activeConfiguration: try binaryValue.field(2)
        )
    }
}
