import AVPlayerABI
import AVFoundation
import CoreVideo
import Foundation

struct RenderedLegibleOutputInfoPayload: AVPBridge {
    let suppressesPlayerRendering: Bool
    let advanceIntervalForDelegateInvocation: Double
    let videoDisplaySize: SizePayload
}

struct RenderedCaptionImagePayload: AVPBridge {
    let x: Double
    let y: Double
    let width: Int
    let height: Int
}

struct RenderedLegibleOutputEventPayload: AVPBridge {
    let event: String
    let itemTime: TimePayload?
    let captionImages: [RenderedCaptionImagePayload]?
}

@available(macOS 15.0, *)
final class AVPPlayerItemRenderedLegibleOutputBox: AVPPlayerItemOutputBox {
    let renderedOutput: AVPlayerItemRenderedLegibleOutput

    init(renderedOutput: AVPlayerItemRenderedLegibleOutput) {
        self.renderedOutput = renderedOutput
    }

    override var output: AVPlayerItemOutput {
        renderedOutput
    }
}

@available(macOS 15.0, *)
final class RenderedLegibleOutputObserverBox: NSObject, AVPlayerItemRenderedLegibleOutputPushDelegate {
    private weak var output: AVPlayerItemRenderedLegibleOutput?
    private let callback: AVPValueObjectsCallback
    private let gate: AVPCallbackGate

    init(
        output: AVPlayerItemRenderedLegibleOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPValueObjectsCallback,
        gate: AVPCallbackGate
    ) {
        self.output = output
        self.callback = callback
        self.gate = gate
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        output?.setDelegate(nil, queue: nil)
        gate.finish()
    }

    func outputSequenceWasFlushed(_ output: AVPlayerItemOutput) {
        gate.deliverValue(
            RenderedLegibleOutputEventPayload(event: "sequence_was_flushed", itemTime: nil, captionImages: nil),
            objects: [],
            to: callback
        )
    }

    func renderedLegibleOutput(
        _ output: AVPlayerItemRenderedLegibleOutput,
        didOutputRenderedCaptionImages captionImages: [AVRenderedCaptionImage],
        forItemTime itemTime: CMTime
    ) {
        let payload = RenderedLegibleOutputEventPayload(
            event: "rendered_caption_images",
            itemTime: encodeTime(itemTime),
            captionImages: captionImages.map(encodeRenderedCaptionImage)
        )
        gate.deliverValue(payload, objects: captionImages.map { $0.pixelBuffer }, to: callback)
    }
}

@_cdecl("av_player_item_rendered_legible_output_create")
public func av_player_item_rendered_legible_output_create(
    _ width: Double,
    _ height: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerItemRenderedLegibleOutput requires macOS 15.0+")
        return nil
    }
    let size = CGSize(width: width, height: height)
    guard width > 0, height > 0 else {
        outErrorMessage?.pointee = ffiString("video display size must be positive")
        return nil
    }
    let output = AVPlayerItemRenderedLegibleOutput(videoDisplay: size)
    return Unmanaged.passRetained(AVPPlayerItemRenderedLegibleOutputBox(renderedOutput: output)).toOpaque()
}

@_cdecl("av_player_item_rendered_legible_output_info_value")
public func av_player_item_rendered_legible_output_info_value(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerItemRenderedLegibleOutput requires macOS 15.0+")
        return nil
    }
    let box = Unmanaged<AVPPlayerItemRenderedLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let payload = RenderedLegibleOutputInfoPayload(
        suppressesPlayerRendering: box.renderedOutput.suppressesPlayerRendering,
        advanceIntervalForDelegateInvocation: box.renderedOutput.advanceIntervalForDelegateInvocation,
        videoDisplaySize: encodeSize(box.renderedOutput.videoDisplaySize)
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_rendered_legible_output_set_advance_interval")
public func av_player_item_rendered_legible_output_set_advance_interval(
    _ outputPtr: UnsafeMutableRawPointer,
    _ interval: Double
) {
    guard #available(macOS 15.0, *) else { return }
    let box = Unmanaged<AVPPlayerItemRenderedLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    box.renderedOutput.advanceIntervalForDelegateInvocation = interval
}

@_cdecl("av_player_item_rendered_legible_output_set_video_display_size")
public func av_player_item_rendered_legible_output_set_video_display_size(
    _ outputPtr: UnsafeMutableRawPointer,
    _ width: Double,
    _ height: Double
) {
    guard #available(macOS 15.0, *) else { return }
    guard width > 0, height > 0 else { return }
    let box = Unmanaged<AVPPlayerItemRenderedLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    box.renderedOutput.videoDisplaySize = CGSize(width: width, height: height)
}

@_cdecl("av_player_item_rendered_legible_output_add_observer")
public func av_player_item_rendered_legible_output_add_observer(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<AVPValue>?,
    _ callback: AVPValueObjectsCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("AVPlayerItemRenderedLegibleOutput requires macOS 15.0+")
        return nil
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing rendered-legible-output observer callback")
        return nil
    }
    let output = Unmanaged<AVPPlayerItemRenderedLegibleOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().renderedOutput
    let observer = RenderedLegibleOutputObserverBox(
        output: output,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_rendered_legible_output_observer_release")
public func av_player_item_rendered_legible_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    if #available(macOS 15.0, *) {
        let observer = Unmanaged<RenderedLegibleOutputObserverBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
    }
}

@available(macOS 15.0, *)
private func encodeRenderedCaptionImage(_ image: AVRenderedCaptionImage) -> RenderedCaptionImagePayload {
    RenderedCaptionImagePayload(
        x: image.position.x,
        y: image.position.y,
        width: CVPixelBufferGetWidth(image.pixelBuffer),
        height: CVPixelBufferGetHeight(image.pixelBuffer)
    )
}

extension RenderedLegibleOutputInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            suppressesPlayerRendering.binaryValue,
            advanceIntervalForDelegateInvocation.binaryValue,
            videoDisplaySize.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            suppressesPlayerRendering: try binaryValue.field(0),
            advanceIntervalForDelegateInvocation: try binaryValue.field(1),
            videoDisplaySize: try binaryValue.field(2)
        )
    }
}

extension RenderedCaptionImagePayload {
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

extension RenderedLegibleOutputEventPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            event.binaryValue,
            itemTime.binaryValue,
            captionImages.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            event: try binaryValue.field(0),
            itemTime: try binaryValue.field(1),
            captionImages: try binaryValue.field(2)
        )
    }
}
