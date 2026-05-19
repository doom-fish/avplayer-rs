import AVFoundation
import Foundation

private struct ContentKeySessionInfoPayload: Codable {
    let keySystem: String
    let storageURL: String?
    let contentProtectionSessionIdentifierBase64: String?
    let recipientCount: Int
}

private struct ContentKeyIdentifierPayload: Codable {
    let kind: String
    let value: String?
    let bytes: [UInt8]?
    let numberValue: Int64?
}

private struct ContentKeyRequestOptionsPayload: Codable {
    let protocolVersions: [Int]?
    let shouldRandomizeDeviceIdentifier: Bool?
    let randomDeviceIdentifierSeed: [UInt8]?
}

private struct ContentKeyRequestInfoPayload: Codable {
    let status: Int32
    let errorMessage: String?
    let identifier: ContentKeyIdentifierPayload?
    let initializationData: [UInt8]?
    let options: ContentKeyRequestOptionsPayload?
    let canProvidePersistableContentKey: Bool
    let renewsExpiringResponseData: Bool
    let hasContentKeySpecifier: Bool
    let hasContentKey: Bool
}

private struct ContentKeySpecifierInfoPayload: Codable {
    let keySystem: String
    let identifier: ContentKeyIdentifierPayload?
    let options: ContentKeyRequestOptionsPayload?
}

private struct ContentKeyInfoPayload: Codable {
    let externalContentProtectionStatus: Int32?
}

private struct ContentKeyBytesPayload: Codable {
    let bytes: [UInt8]?
}

private struct ContentKeySessionEventPayload: Codable {
    let event: String
    let requestPtr: UInt64?
    let keyRequestPtrs: [UInt64]?
    let contentKeyPtr: UInt64?
    let initializationData: [UInt8]?
    let persistableContentKey: [UInt8]?
    let keyIdentifier: ContentKeyIdentifierPayload?
    let errorMessage: String?
    let retryReason: String?
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

private func avpContentKeyIdentifierPayload(from identifier: Any?) -> ContentKeyIdentifierPayload? {
    guard let identifier else { return nil }
    switch identifier {
    case let string as String:
        return ContentKeyIdentifierPayload(kind: "string", value: string, bytes: nil, numberValue: nil)
    case let string as NSString:
        return ContentKeyIdentifierPayload(kind: "string", value: string as String, bytes: nil, numberValue: nil)
    case let url as URL:
        return ContentKeyIdentifierPayload(kind: "url", value: url.absoluteString, bytes: nil, numberValue: nil)
    case let url as NSURL:
        return ContentKeyIdentifierPayload(kind: "url", value: (url as URL).absoluteString, bytes: nil, numberValue: nil)
    case let data as Data:
        return ContentKeyIdentifierPayload(kind: "data", value: nil, bytes: [UInt8](data), numberValue: nil)
    case let data as NSData:
        return ContentKeyIdentifierPayload(kind: "data", value: nil, bytes: [UInt8](data as Data), numberValue: nil)
    case let number as NSNumber:
        return ContentKeyIdentifierPayload(kind: "number", value: nil, bytes: nil, numberValue: number.int64Value)
    default:
        return ContentKeyIdentifierPayload(
            kind: "string",
            value: String(describing: identifier),
            bytes: nil,
            numberValue: nil
        )
    }
}

private func avpContentKeyIdentifier(from payload: ContentKeyIdentifierPayload) throws -> Any {
    switch payload.kind {
    case "string":
        return payload.value ?? ""
    case "url":
        guard let value = payload.value, let url = URL(string: value) else {
            throw BridgeError.message("content-key identifier URL was invalid")
        }
        return url
    case "data":
        return Data(payload.bytes ?? [])
    case "number":
        return NSNumber(value: payload.numberValue ?? 0)
    default:
        throw BridgeError.message("unsupported content-key identifier kind: \(payload.kind)")
    }
}

private func avpNSNumberArray(from value: Any?) -> [Int]? {
    if let numbers = value as? [NSNumber] {
        return numbers.map(\.intValue)
    }
    if let integers = value as? [Int] {
        return integers
    }
    return nil
}

private func avpBool(from value: Any?) -> Bool? {
    if let number = value as? NSNumber {
        return number.boolValue
    }
    if let bool = value as? Bool {
        return bool
    }
    return nil
}

private func avpDataBytes(from value: Any?) -> [UInt8]? {
    if let data = value as? Data {
        return [UInt8](data)
    }
    if let data = value as? NSData {
        return [UInt8](data as Data)
    }
    if let bytes = value as? [UInt8] {
        return bytes
    }
    return nil
}

private func avpContentKeyRequestOptionsPayload(
    from options: [String: Any]?
) -> ContentKeyRequestOptionsPayload? {
    guard let options else { return nil }
    let protocolVersions = avpNSNumberArray(from: options[AVContentKeyRequestProtocolVersionsKey])
    let shouldRandomizeDeviceIdentifier: Bool? = {
        if #available(macOS 26.0, *) {
            return avpBool(from: options[AVContentKeyRequestShouldRandomizeDeviceIdentifierKey])
        }
        return nil
    }()
    let randomDeviceIdentifierSeed: [UInt8]? = {
        if #available(macOS 26.0, *) {
            return avpDataBytes(from: options[AVContentKeyRequestRandomDeviceIdentifierSeedKey])
        }
        return nil
    }()
    if protocolVersions == nil,
        shouldRandomizeDeviceIdentifier == nil,
        randomDeviceIdentifierSeed == nil {
        return nil
    }
    return ContentKeyRequestOptionsPayload(
        protocolVersions: protocolVersions,
        shouldRandomizeDeviceIdentifier: shouldRandomizeDeviceIdentifier,
        randomDeviceIdentifierSeed: randomDeviceIdentifierSeed
    )
}

private func avpContentKeyRequestOptions(
    from payload: ContentKeyRequestOptionsPayload?
) throws -> [String: Any]? {
    guard let payload else { return nil }
    var options: [String: Any] = [:]
    if let protocolVersions = payload.protocolVersions, !protocolVersions.isEmpty {
        options[AVContentKeyRequestProtocolVersionsKey] = protocolVersions.map(NSNumber.init(value:))
    }
    if let shouldRandomizeDeviceIdentifier = payload.shouldRandomizeDeviceIdentifier {
        guard #available(macOS 26.0, *) else {
            throw BridgeError.message("randomized content-key identifiers require macOS 26.0")
        }
        options[AVContentKeyRequestShouldRandomizeDeviceIdentifierKey] = NSNumber(value: shouldRandomizeDeviceIdentifier)
    }
    if let randomDeviceIdentifierSeed = payload.randomDeviceIdentifierSeed {
        guard #available(macOS 26.0, *) else {
            throw BridgeError.message("randomized content-key identifiers require macOS 26.0")
        }
        options[AVContentKeyRequestRandomDeviceIdentifierSeedKey] = Data(randomDeviceIdentifierSeed)
    }
    return options.isEmpty ? nil : options
}

private func avpContentKeyRequestRetryReasonString(
    _ reason: AVContentKeyRequest.RetryReason
) -> String {
    switch reason {
    case .timedOut:
        return "timed_out"
    case .receivedResponseWithExpiredLease:
        return "received_response_with_expired_lease"
    case .receivedObsoleteContentKey:
        return "received_obsolete_content_key"
    default:
        return reason.rawValue
    }
}

private func avpRetainedPointerValue(_ object: AnyObject) -> UInt64 {
    UInt64(UInt(bitPattern: Unmanaged.passRetained(object).toOpaque()))
}

private final class ContentKeySessionObserverBox: NSObject, AVContentKeySessionDelegate {
    private weak var session: AVContentKeySession?
    private let callback: AVPBoolJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        session: AVContentKeySession,
        queue: DispatchQueue?,
        callback: @escaping AVPBoolJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.session = session
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        session.setDelegate(self, queue: queue ?? DispatchQueue(label: "avplayer.content-key-session"))
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        session?.setDelegate(nil, queue: nil)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    private func send(_ payload: ContentKeySessionEventPayload) -> Bool {
        guard !disposed else { return false }
        guard let json = try? avpEncodeJSON(payload) else {
            return false
        }
        return json.withCString { callback(userData, $0) }
    }

    func contentKeySession(_ session: AVContentKeySession, didProvide keyRequest: AVContentKeyRequest) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "requested",
                requestPtr: avpRetainedPointerValue(keyRequest),
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    func contentKeySession(
        _ session: AVContentKeySession,
        didProvideRenewingContentKeyRequest keyRequest: AVContentKeyRequest
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "renewing",
                requestPtr: avpRetainedPointerValue(keyRequest),
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    func contentKeySession(
        _ session: AVContentKeySession,
        didProvide keyRequest: AVPersistableContentKeyRequest
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "persistable",
                requestPtr: avpRetainedPointerValue(keyRequest),
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    func contentKeySession(
        _ session: AVContentKeySession,
        didUpdatePersistableContentKey persistableContentKey: Data,
        forContentKeyIdentifier keyIdentifier: Any
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "updated_persistable_content_key",
                requestPtr: nil,
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: [UInt8](persistableContentKey),
                keyIdentifier: avpContentKeyIdentifierPayload(from: keyIdentifier),
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    func contentKeySession(
        _ session: AVContentKeySession,
        contentKeyRequest keyRequest: AVContentKeyRequest,
        didFailWithError err: any Error
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "failed",
                requestPtr: avpRetainedPointerValue(keyRequest),
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: err.localizedDescription,
                retryReason: nil
            )
        )
    }

    func contentKeySession(
        _ session: AVContentKeySession,
        shouldRetry keyRequest: AVContentKeyRequest,
        reason retryReason: AVContentKeyRequest.RetryReason
    ) -> Bool {
        send(
            ContentKeySessionEventPayload(
                event: "retry_requested",
                requestPtr: avpRetainedPointerValue(keyRequest),
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: avpContentKeyRequestRetryReasonString(retryReason)
            )
        )
    }

    func contentKeySession(
        _ session: AVContentKeySession,
        contentKeyRequestDidSucceed keyRequest: AVContentKeyRequest
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "succeeded",
                requestPtr: avpRetainedPointerValue(keyRequest),
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    func contentKeySessionContentProtectionSessionIdentifierDidChange(
        _ session: AVContentKeySession
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "content_protection_session_identifier_did_change",
                requestPtr: nil,
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    func contentKeySessionDidGenerateExpiredSessionReport(_ session: AVContentKeySession) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "expired_session_report_generated",
                requestPtr: nil,
                keyRequestPtrs: nil,
                contentKeyPtr: nil,
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    @available(macOS 14.4, *)
    func contentKeySession(
        _ session: AVContentKeySession,
        externalProtectionStatusDidChangeFor contentKey: AVContentKey
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "external_protection_status_did_change",
                requestPtr: nil,
                keyRequestPtrs: nil,
                contentKeyPtr: avpRetainedPointerValue(contentKey),
                initializationData: nil,
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
    }

    @available(macOS 14.4, *)
    func contentKeySession(
        _ session: AVContentKeySession,
        didProvide keyRequests: [AVContentKeyRequest],
        forInitializationData initializationData: Data?
    ) {
        _ = send(
            ContentKeySessionEventPayload(
                event: "requested_collection",
                requestPtr: nil,
                keyRequestPtrs: keyRequests.map(avpRetainedPointerValue),
                contentKeyPtr: nil,
                initializationData: initializationData.map([UInt8].init),
                persistableContentKey: nil,
                keyIdentifier: nil,
                errorMessage: nil,
                retryReason: nil
            )
        )
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

@_cdecl("av_content_key_session_add_observer")
public func av_content_key_session_add_observer(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPBoolJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing content-key-session callback")
        return nil
    }
    let session = Unmanaged<AVContentKeySession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let observer = ContentKeySessionObserverBox(
        session: session,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_content_key_session_observer_release")
public func av_content_key_session_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<ContentKeySessionObserverBox>.fromOpaque(observerPtr).release()
}

@_cdecl("av_content_key_session_process_content_key_request")
public func av_content_key_session_process_content_key_request(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ identifierJson: UnsafePointer<CChar>?,
    _ initializationDataBytes: UnsafePointer<UInt8>?,
    _ initializationDataCount: Int,
    _ optionsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = Unmanaged<AVContentKeySession>.fromOpaque(sessionPtr).takeUnretainedValue()
    do {
        let identifier: Any?
        if let identifierJson {
            let payload = try avpDecodeJSON(identifierJson, as: ContentKeyIdentifierPayload.self)
            identifier = try avpContentKeyIdentifier(from: payload)
        } else {
            identifier = nil
        }
        let initializationData = initializationDataBytes.map {
            Data(bytes: $0, count: initializationDataCount)
        }
        if identifier == nil, initializationData == nil {
            throw BridgeError.message("content-key requests require an identifier, initialization data, or both")
        }
        let optionsPayload = try optionsJson.map {
            try avpDecodeJSON($0, as: ContentKeyRequestOptionsPayload.self)
        }
        let options = try avpContentKeyRequestOptions(from: optionsPayload)
        session.processContentKeyRequest(
            withIdentifier: identifier,
            initializationData: initializationData,
            options: options
        )
        return AVP_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVP_INVALID_ARGUMENT
    }
}

@_cdecl("av_content_key_session_renew_expiring_response_data_for_request")
public func av_content_key_session_renew_expiring_response_data_for_request(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = Unmanaged<AVContentKeySession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    guard session.keySystem == .fairPlayStreaming else {
        outErrorMessage?.pointee = ffiString("content-key renewal requires FairPlay Streaming")
        return AVP_INVALID_ARGUMENT
    }
    session.renewExpiringResponseData(for: request)
    return AVP_OK
}

@_cdecl("av_content_key_request_info_json")
public func av_content_key_request_info_json(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let payload = ContentKeyRequestInfoPayload(
        status: Int32(request.status.rawValue),
        errorMessage: request.error?.localizedDescription,
        identifier: avpContentKeyIdentifierPayload(from: request.identifier),
        initializationData: request.initializationData.map([UInt8].init),
        options: {
            if #available(macOS 10.14.4, *) {
                return avpContentKeyRequestOptionsPayload(from: request.options)
            }
            return nil
        }(),
        canProvidePersistableContentKey: request.canProvidePersistableContentKey,
        renewsExpiringResponseData: request.renewsExpiringResponseData,
        hasContentKeySpecifier: {
            if #available(macOS 11.3, *) {
                return true
            }
            return false
        }(),
        hasContentKey: {
            if #available(macOS 11.3, *) {
                return request.contentKey != nil
            }
            return false
        }()
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_content_key_request_copy_content_key_specifier")
public func av_content_key_request_copy_content_key_specifier(
    _ requestPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 11.3, *) else { return nil }
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    return Unmanaged.passRetained(request.contentKeySpecifier).toOpaque()
}

@_cdecl("av_content_key_request_copy_content_key")
public func av_content_key_request_copy_content_key(
    _ requestPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 11.3, *) else { return nil }
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    guard let contentKey = request.contentKey else { return nil }
    return Unmanaged.passRetained(contentKey).toOpaque()
}

@_cdecl("av_content_key_request_make_streaming_content_key_request_data_json")
public func av_content_key_request_make_streaming_content_key_request_data_json(
    _ requestPtr: UnsafeMutableRawPointer,
    _ appIdentifierBytes: UnsafePointer<UInt8>?,
    _ appIdentifierCount: Int,
    _ contentIdentifierBytes: UnsafePointer<UInt8>?,
    _ contentIdentifierCount: Int,
    _ optionsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let appIdentifier = appIdentifierBytes.map { Data(bytes: $0, count: appIdentifierCount) } ?? Data()
    let contentIdentifier = contentIdentifierBytes.map { Data(bytes: $0, count: contentIdentifierCount) }
    do {
        let optionsPayload = try optionsJson.map {
            try avpDecodeJSON($0, as: ContentKeyRequestOptionsPayload.self)
        }
        let options = try avpContentKeyRequestOptions(from: optionsPayload)
        let semaphore = DispatchSemaphore(value: 0)
        var returnedData: Data?
        var returnedError: Error?
        request.makeStreamingContentKeyRequestData(
            forApp: appIdentifier,
            contentIdentifier: contentIdentifier,
            options: options
        ) { data, error in
            returnedData = data
            returnedError = error
            semaphore.signal()
        }
        if semaphore.wait(timeout: .now() + .seconds(30)) == .timedOut {
            throw BridgeError.message("timed out waiting for streaming content-key request data")
        }
        if let returnedError {
            throw returnedError
        }
        return ffiString(
            try avpEncodeJSON(
                ContentKeyBytesPayload(bytes: returnedData.map([UInt8].init))
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_content_key_request_process_content_key_response")
public func av_content_key_request_process_content_key_response(
    _ requestPtr: UnsafeMutableRawPointer,
    _ responsePtr: UnsafeMutableRawPointer?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let responsePtr else {
        outErrorMessage?.pointee = ffiString("missing content-key response")
        return AVP_INVALID_ARGUMENT
    }
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let response = Unmanaged<AVContentKeyResponse>.fromOpaque(responsePtr).takeUnretainedValue()
    request.processContentKeyResponse(response)
    return AVP_OK
}

@_cdecl("av_content_key_request_process_content_key_response_error")
public func av_content_key_request_process_content_key_response_error(
    _ requestPtr: UnsafeMutableRawPointer,
    _ messagePtr: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let message = messagePtr.map { String(cString: $0) } ?? "content-key response failed"
    let error = NSError(
        domain: "avplayer.content-key-session",
        code: -1,
        userInfo: [NSLocalizedDescriptionKey: message]
    )
    request.processContentKeyResponseError(error)
    _ = outErrorMessage
    return AVP_OK
}

@_cdecl("av_content_key_request_request_persistable_content_key")
public func av_content_key_request_request_persistable_content_key(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("persistable content keys require macOS 10.15")
        return AVP_INVALID_ARGUMENT
    }
    let request = Unmanaged<AVContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    do {
        try request.respondByRequestingPersistableContentKeyRequest()
        return AVP_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVP_OPERATION_FAILED
    }
}

@_cdecl("av_persistable_content_key_request_persistable_content_key_json")
public func av_persistable_content_key_request_persistable_content_key_json(
    _ requestPtr: UnsafeMutableRawPointer,
    _ keyVendorResponseBytes: UnsafePointer<UInt8>?,
    _ keyVendorResponseCount: Int,
    _ optionsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("persistable content keys require macOS 10.15")
        return nil
    }
    let request = Unmanaged<AVPersistableContentKeyRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let keyVendorResponse = keyVendorResponseBytes.map {
        Data(bytes: $0, count: keyVendorResponseCount)
    } ?? Data()
    do {
        let optionsPayload = try optionsJson.map {
            try avpDecodeJSON($0, as: ContentKeyRequestOptionsPayload.self)
        }
        let options = try avpContentKeyRequestOptions(from: optionsPayload)
        let persistableContentKey = try request.persistableContentKey(
            fromKeyVendorResponse: keyVendorResponse,
            options: options
        )
        return ffiString(
            try avpEncodeJSON(
                ContentKeyBytesPayload(bytes: [UInt8](persistableContentKey))
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_content_key_response_create_fair_play_streaming")
public func av_content_key_response_create_fair_play_streaming(
    _ bytes: UnsafePointer<UInt8>?,
    _ count: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let data = bytes.map { Data(bytes: $0, count: count) } ?? Data()
    _ = outErrorMessage
    return Unmanaged.passRetained(
        AVContentKeyResponse(fairPlayStreamingKeyResponseData: data)
    ).toOpaque()
}

@_cdecl("av_content_key_response_create_clear_key")
public func av_content_key_response_create_clear_key(
    _ keyDataBytes: UnsafePointer<UInt8>?,
    _ keyDataCount: Int,
    _ initializationVectorBytes: UnsafePointer<UInt8>?,
    _ initializationVectorCount: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.13, *) else {
        outErrorMessage?.pointee = ffiString("clear-key content responses require macOS 10.13")
        return nil
    }
    let keyData = keyDataBytes.map { Data(bytes: $0, count: keyDataCount) } ?? Data()
    let initializationVector = initializationVectorBytes.map {
        Data(bytes: $0, count: initializationVectorCount)
    }
    return Unmanaged.passRetained(
        AVContentKeyResponse(clearKeyData: keyData, initializationVector: initializationVector)
    ).toOpaque()
}

@_cdecl("av_content_key_response_create_authorization_token")
public func av_content_key_response_create_authorization_token(
    _ bytes: UnsafePointer<UInt8>?,
    _ count: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("authorization-token content responses require macOS 10.15")
        return nil
    }
    let data = bytes.map { Data(bytes: $0, count: count) } ?? Data()
    return Unmanaged.passRetained(
        AVContentKeyResponse(authorizationTokenData: data)
    ).toOpaque()
}

@_cdecl("av_content_key_specifier_create")
public func av_content_key_specifier_create(
    _ keySystemPtr: UnsafePointer<CChar>,
    _ identifierJson: UnsafePointer<CChar>,
    _ optionsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let keySystem = avpContentKeySystem(from: String(cString: keySystemPtr))
    do {
        let identifierPayload = try avpDecodeJSON(identifierJson, as: ContentKeyIdentifierPayload.self)
        let identifier = try avpContentKeyIdentifier(from: identifierPayload)
        let optionsPayload = try optionsJson.map {
            try avpDecodeJSON($0, as: ContentKeyRequestOptionsPayload.self)
        }
        let options = try avpContentKeyRequestOptions(from: optionsPayload) ?? [:]
        return Unmanaged.passRetained(
            AVContentKeySpecifier(forKeySystem: keySystem, identifier: identifier, options: options)
        ).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_content_key_specifier_info_json")
public func av_content_key_specifier_info_json(
    _ specifierPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let specifier = Unmanaged<AVContentKeySpecifier>.fromOpaque(specifierPtr).takeUnretainedValue()
    let payload = ContentKeySpecifierInfoPayload(
        keySystem: avpContentKeySystemString(specifier.keySystem),
        identifier: avpContentKeyIdentifierPayload(from: specifier.identifier),
        options: avpContentKeyRequestOptionsPayload(from: specifier.options)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_content_key_info_json")
public func av_content_key_info_json(
    _ contentKeyPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let contentKey = Unmanaged<AVContentKey>.fromOpaque(contentKeyPtr).takeUnretainedValue()
    let payload = ContentKeyInfoPayload(
        externalContentProtectionStatus: {
            if #available(macOS 14.4, *) {
                return Int32(contentKey.externalContentProtectionStatus.rawValue)
            }
            return nil
        }()
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_content_key_copy_content_key_specifier")
public func av_content_key_copy_content_key_specifier(
    _ contentKeyPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let contentKey = Unmanaged<AVContentKey>.fromOpaque(contentKeyPtr).takeUnretainedValue()
    return Unmanaged.passRetained(contentKey.contentKeySpecifier).toOpaque()
}

@_cdecl("av_content_key_revoke")
public func av_content_key_revoke(
    _ contentKeyPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.4, *) else {
        outErrorMessage?.pointee = ffiString("revoking content keys requires macOS 14.4")
        return AVP_INVALID_ARGUMENT
    }
    let contentKey = Unmanaged<AVContentKey>.fromOpaque(contentKeyPtr).takeUnretainedValue()
    contentKey.revoke()
    return AVP_OK
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
