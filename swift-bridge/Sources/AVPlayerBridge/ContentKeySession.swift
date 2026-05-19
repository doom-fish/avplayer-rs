import AVFoundation
import Foundation

private struct ContentKeySessionInfoPayload: Codable {
    let keySystem: String
    let storageURL: String?
    let contentProtectionSessionIdentifierBase64: String?
    let recipientCount: Int
}

private func avpContentKeySystem(from raw: String) -> AVContentKeySystem {
    switch raw {
    case "fair_play_streaming":
        return .fairPlayStreaming
    case "clear_key":
        return .clearKey
    case "authorization_token":
        return .authorizationToken
    default:
        return AVContentKeySystem(rawValue: raw)
    }
}

private func avpContentKeySystemString(_ keySystem: AVContentKeySystem) -> String {
    switch keySystem {
    case .fairPlayStreaming:
        return "fair_play_streaming"
    case .clearKey:
        return "clear_key"
    case .authorizationToken:
        return "authorization_token"
    default:
        return keySystem.rawValue
    }
}

@_cdecl("av_url_asset_may_require_content_keys_for_media_data_processing")
public func av_url_asset_may_require_content_keys_for_media_data_processing(
    _ assetPtr: UnsafeMutableRawPointer
) -> Bool {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard let urlAsset = asset as? AVURLAsset else { return false }
    return urlAsset.mayRequireContentKeysForMediaDataProcessing
}

@_cdecl("av_content_key_session_create")
public func av_content_key_session_create(
    _ keySystemPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.13, *) else {
        outErrorMessage?.pointee = ffiString("AVContentKeySession requires macOS 10.13")
        return nil
    }
    let keySystem = avpContentKeySystem(from: String(cString: keySystemPtr))
    return Unmanaged.passRetained(AVContentKeySession(keySystem: keySystem)).toOpaque()
}

@_cdecl("av_content_key_session_create_with_storage_directory")
public func av_content_key_session_create_with_storage_directory(
    _ keySystemPtr: UnsafePointer<CChar>,
    _ pathPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.13, *) else {
        outErrorMessage?.pointee = ffiString("AVContentKeySession requires macOS 10.13")
        return nil
    }
    let keySystem = avpContentKeySystem(from: String(cString: keySystemPtr))
    let path = String(cString: pathPtr)
    return Unmanaged.passRetained(
        AVContentKeySession(keySystem: keySystem, storageDirectoryAt: URL(fileURLWithPath: path))
    ).toOpaque()
}

@_cdecl("av_content_key_session_info_json")
public func av_content_key_session_info_json(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.13, *) else {
        outErrorMessage?.pointee = ffiString("AVContentKeySession requires macOS 10.13")
        return nil
    }
    let session = Unmanaged<AVContentKeySession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let payload = ContentKeySessionInfoPayload(
        keySystem: avpContentKeySystemString(session.keySystem),
        storageURL: session.storageURL?.absoluteString,
        contentProtectionSessionIdentifierBase64: session.contentProtectionSessionIdentifier?.base64EncodedString(),
        recipientCount: session.contentKeyRecipients.count
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_content_key_session_add_content_key_recipient")
public func av_content_key_session_add_content_key_recipient(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ recipientPtr: UnsafeMutableRawPointer
) {
    guard #available(macOS 10.13, *) else { return }
    let session = Unmanaged<AVContentKeySession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let recipient = Unmanaged<AVAsset>.fromOpaque(recipientPtr).takeUnretainedValue()
    if let urlAsset = recipient as? AVURLAsset {
        session.addContentKeyRecipient(urlAsset)
    }
}

@_cdecl("av_content_key_session_remove_content_key_recipient")
public func av_content_key_session_remove_content_key_recipient(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ recipientPtr: UnsafeMutableRawPointer
) {
    guard #available(macOS 10.13, *) else { return }
    let session = Unmanaged<AVContentKeySession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let recipient = Unmanaged<AVAsset>.fromOpaque(recipientPtr).takeUnretainedValue()
    if let urlAsset = recipient as? AVURLAsset {
        session.removeContentKeyRecipient(urlAsset)
    }
}

@_cdecl("av_content_key_session_expire")
public func av_content_key_session_expire(_ sessionPtr: UnsafeMutableRawPointer) {
    guard #available(macOS 10.13, *) else { return }
    let session = Unmanaged<AVContentKeySession>.fromOpaque(sessionPtr).takeUnretainedValue()
    session.expire()
}
