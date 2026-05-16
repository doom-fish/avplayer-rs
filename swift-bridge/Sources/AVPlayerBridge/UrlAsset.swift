import AVFoundation
import Foundation

@_cdecl("av_url_asset_create")
public func av_url_asset_create(
    _ urlPtr: UnsafePointer<CChar>,
    _ isFileURL: Bool,
    _ preferPreciseDuration: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let urlString = String(cString: urlPtr)
    let url = isFileURL ? URL(fileURLWithPath: urlString) : URL(string: urlString)
    guard let url else {
        outErrorMessage?.pointee = ffiString("invalid URL: \(urlString)")
        return nil
    }

    let asset = AVURLAsset(
        url: url,
        options: [AVURLAssetPreferPreciseDurationAndTimingKey: preferPreciseDuration]
    )
    return Unmanaged.passRetained(asset).toOpaque()
}
