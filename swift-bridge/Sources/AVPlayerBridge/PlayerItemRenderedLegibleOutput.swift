import AVFoundation
import CoreVideo
import Foundation

struct RenderedLegibleOutputInfoPayload: Codable {
    let suppressesPlayerRendering: Bool
    let advanceIntervalForDelegateInvocation: Double
    let videoDisplaySize: SizePayload
}

struct RenderedCaptionImagePayload: Codable {
    let x: Double
    let y: Double
    let width: Int
    let height: Int
}

struct RenderedLegibleOutputEventPayload: Codable {
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
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        output: AVPlayerItemRenderedLegibleOutput,
        queue: DispatchQueue?,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.output = output
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        output.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        output?.setDelegate(nil, queue: nil)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    func outputSequenceWasFlushed(_ output: AVPlayerItemOutput) {
        send(RenderedLegibleOutputEventPayload(event: "sequence_was_flushed", itemTime: nil, captionImages: nil))
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
        send(payload)
    }

    private func send(_ payload: RenderedLegibleOutputEventPayload) {
        guard !disposed else { return }
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
    }
}

@_cdecl("av_player_item_rendered_legible_output_create")
public func av_player_item_rendered_legible_output_create(
    _ width: Double,
    _ height: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
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

@_cdecl("av_player_item_rendered_legible_output_info_json")
public func av_player_item_rendered_legible_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
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
        return ffiString(try avpEncodeJSON(payload))
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
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
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
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_item_rendered_legible_output_observer_release")
public func av_player_item_rendered_legible_output_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    if #available(macOS 15.0, *) {
        Unmanaged<RenderedLegibleOutputObserverBox>.fromOpaque(observerPtr).release()
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
