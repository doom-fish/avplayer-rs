import AVPlayerABI
import AVFoundation
import Foundation

private struct AssetResourceLoaderInfoPayload: AVPBridge {
    let hasDelegate: Bool
    let preloadsEligibleContentKeys: Bool
    let sendsCommonMediaClientDataAsHTTPHeaders: Bool?
}

private struct AssetResourceLoadingRequestPayload: AVPBridge {
    let requestURL: String?
    let requestMethod: String?
    let requestHeaders: [String: String]?
    let finished: Bool
    let cancelled: Bool
    let hasContentInformationRequest: Bool
    let hasDataRequest: Bool
    let hasRequestor: Bool
}

private struct AssetResourceLoadingContentInformationRequestPayload: AVPBridge {
    let contentType: String?
    let allowedContentTypes: [String]?
    let contentLength: Int64
    let byteRangeAccessSupported: Bool
    let renewalDateISO8601: String?
    let entireLengthAvailableOnDemand: Bool?
}

private struct AssetResourceLoadingDataRequestPayload: AVPBridge {
    let requestedOffset: Int64
    let requestedLength: Int64
    let requestsAllDataToEndOfResource: Bool?
    let currentOffset: Int64
}

private struct AssetResourceLoadingRequestorPayload: AVPBridge {
    let providesExpiredSessionReports: Bool
}

private final class AssetResourceLoaderDelegateBox: NSObject, AVAssetResourceLoaderDelegate {
    private weak var loader: AVAssetResourceLoader?
    private let callback: AVPBoolObjectCallback
    private let gate: AVPCallbackGate

    init(
        loader: AVAssetResourceLoader,
        queue: DispatchQueue?,
        callback: @escaping AVPBoolObjectCallback,
        gate: AVPCallbackGate
    ) {
        self.loader = loader
        self.callback = callback
        self.gate = gate
        super.init()
        loader.setDelegate(self, queue: queue ?? DispatchQueue(label: "avplayer.resource-loader"))
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard gate.close() else { return }
        loader?.setDelegate(nil, queue: nil)
        gate.finish()
    }

    private func send(event: String, object: AnyObject) -> Bool {
        gate.run(false) { context in
            let objectPtr = Unmanaged.passRetained(object).toOpaque()
            return event.withBridgeString { callback(context, $0, objectPtr) }
        }
    }

    func resourceLoader(
        _ resourceLoader: AVAssetResourceLoader,
        shouldWaitForLoadingOfRequestedResource loadingRequest: AVAssetResourceLoadingRequest
    ) -> Bool {
        send(event: "loading_requested", object: loadingRequest)
    }

    func resourceLoader(
        _ resourceLoader: AVAssetResourceLoader,
        shouldWaitForRenewalOfRequestedResource renewalRequest: AVAssetResourceRenewalRequest
    ) -> Bool {
        send(event: "renewal_requested", object: renewalRequest)
    }

    func resourceLoader(
        _ resourceLoader: AVAssetResourceLoader,
        didCancel loadingRequest: AVAssetResourceLoadingRequest
    ) {
        _ = send(event: "loading_cancelled", object: loadingRequest)
    }
}

@_cdecl("av_url_asset_copy_resource_loader")
public func av_url_asset_copy_resource_loader(
    _ assetPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard let urlAsset = asset as? AVURLAsset else { return nil }
    return Unmanaged.passRetained(urlAsset.resourceLoader).toOpaque()
}

@_cdecl("av_asset_resource_loader_info_value")
public func av_asset_resource_loader_info_value(
    _ loaderPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let loader = Unmanaged<AVAssetResourceLoader>.fromOpaque(loaderPtr).takeUnretainedValue()
    let payload = AssetResourceLoaderInfoPayload(
        hasDelegate: loader.delegate != nil,
        preloadsEligibleContentKeys: loader.preloadsEligibleContentKeys,
        sendsCommonMediaClientDataAsHTTPHeaders: {
            if #available(macOS 15.0, *) {
                return loader.sendsCommonMediaClientDataAsHTTPHeaders
            }
            return nil
        }()
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_resource_loader_set_preloads_eligible_content_keys")
public func av_asset_resource_loader_set_preloads_eligible_content_keys(
    _ loaderPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let loader = Unmanaged<AVAssetResourceLoader>.fromOpaque(loaderPtr).takeUnretainedValue()
    loader.preloadsEligibleContentKeys = enabled
}

@_cdecl("av_asset_resource_loader_set_sends_common_media_client_data_as_http_headers")
public func av_asset_resource_loader_set_sends_common_media_client_data_as_http_headers(
    _ loaderPtr: UnsafeMutableRawPointer,
    _ enabled: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("sendsCommonMediaClientDataAsHTTPHeaders requires macOS 15.0")
        return AVP_INVALID_ARGUMENT
    }
    let loader = Unmanaged<AVAssetResourceLoader>.fromOpaque(loaderPtr).takeUnretainedValue()
    loader.sendsCommonMediaClientDataAsHTTPHeaders = enabled
    return AVP_OK
}

@_cdecl("av_asset_resource_loader_add_delegate")
public func av_asset_resource_loader_add_delegate(
    _ loaderPtr: UnsafeMutableRawPointer,
    _ queueLabel: UnsafePointer<AVPValue>?,
    _ callback: AVPBoolObjectCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let gate = AVPCallbackGate(context: userData, release: dropUserData)
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing resource-loader callback")
        return nil
    }
    let loader = Unmanaged<AVAssetResourceLoader>.fromOpaque(loaderPtr).takeUnretainedValue()
    let observer = AssetResourceLoaderDelegateBox(
        loader: loader,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        gate: gate
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_asset_resource_loader_delegate_release")
public func av_asset_resource_loader_delegate_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    let observer = Unmanaged<AssetResourceLoaderDelegateBox>.fromOpaque(observerPtr)
    observer.takeUnretainedValue().dispose()
    observer.release()
}

@_cdecl("av_asset_resource_loading_request_info_value")
public func av_asset_resource_loading_request_info_value(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let request = Unmanaged<AVAssetResourceLoadingRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let payload = AssetResourceLoadingRequestPayload(
        requestURL: request.request.url?.absoluteString,
        requestMethod: request.request.httpMethod,
        requestHeaders: request.request.allHTTPHeaderFields,
        finished: request.isFinished,
        cancelled: request.isCancelled,
        hasContentInformationRequest: request.contentInformationRequest != nil,
        hasDataRequest: request.dataRequest != nil,
        hasRequestor: {
            if #available(macOS 10.14, *) {
                return true
            }
            return false
        }()
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_resource_loading_request_finish_loading")
public func av_asset_resource_loading_request_finish_loading(
    _ requestPtr: UnsafeMutableRawPointer
) {
    let request = Unmanaged<AVAssetResourceLoadingRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.finishLoading()
}

@_cdecl("av_asset_resource_loading_request_finish_loading_with_error")
public func av_asset_resource_loading_request_finish_loading_with_error(
    _ requestPtr: UnsafeMutableRawPointer,
    _ messagePtr: UnsafePointer<AVPValue>?
) {
    let request = Unmanaged<AVAssetResourceLoadingRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    if let messagePtr {
        let message = String(bridge: messagePtr)
        let error = NSError(
            domain: "avplayer.resource-loader",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: message]
        )
        request.finishLoading(with: error)
    } else {
        request.finishLoading(with: nil)
    }
}

@_cdecl("av_asset_resource_loading_request_copy_content_information_request")
public func av_asset_resource_loading_request_copy_content_information_request(
    _ requestPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let request = Unmanaged<AVAssetResourceLoadingRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    guard let contentInformationRequest = request.contentInformationRequest else { return nil }
    return Unmanaged.passRetained(contentInformationRequest).toOpaque()
}

@_cdecl("av_asset_resource_loading_request_copy_data_request")
public func av_asset_resource_loading_request_copy_data_request(
    _ requestPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let request = Unmanaged<AVAssetResourceLoadingRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    guard let dataRequest = request.dataRequest else { return nil }
    return Unmanaged.passRetained(dataRequest).toOpaque()
}

@_cdecl("av_asset_resource_loading_request_copy_requestor")
public func av_asset_resource_loading_request_copy_requestor(
    _ requestPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.14, *) else { return nil }
    let request = Unmanaged<AVAssetResourceLoadingRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    return Unmanaged.passRetained(request.requestor).toOpaque()
}

@_cdecl("av_asset_resource_loading_content_information_request_info_value")
public func av_asset_resource_loading_content_information_request_info_value(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let payload = AssetResourceLoadingContentInformationRequestPayload(
        contentType: request.contentType,
        allowedContentTypes: {
            if #available(macOS 10.13.2, *) {
                return request.allowedContentTypes
            }
            return nil
        }(),
        contentLength: request.contentLength,
        byteRangeAccessSupported: request.isByteRangeAccessSupported,
        renewalDateISO8601: avpISO8601String(request.renewalDate),
        entireLengthAvailableOnDemand: {
            if #available(macOS 13.0, *) {
                return request.isEntireLengthAvailableOnDemand
            }
            return nil
        }()
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_resource_loading_content_information_request_set_content_type")
public func av_asset_resource_loading_content_information_request_set_content_type(
    _ requestPtr: UnsafeMutableRawPointer,
    _ contentTypePtr: UnsafePointer<AVPValue>?
) {
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.contentType = contentTypePtr.map { String(bridge: $0) }
}

@_cdecl("av_asset_resource_loading_content_information_request_set_content_length")
public func av_asset_resource_loading_content_information_request_set_content_length(
    _ requestPtr: UnsafeMutableRawPointer,
    _ contentLength: Int64
) {
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.contentLength = contentLength
}

@_cdecl("av_asset_resource_loading_content_information_request_set_byte_range_access_supported")
public func av_asset_resource_loading_content_information_request_set_byte_range_access_supported(
    _ requestPtr: UnsafeMutableRawPointer,
    _ supported: Bool
) {
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.isByteRangeAccessSupported = supported
}

@_cdecl("av_asset_resource_loading_content_information_request_set_renewal_date_iso8601")
public func av_asset_resource_loading_content_information_request_set_renewal_date_iso8601(
    _ requestPtr: UnsafeMutableRawPointer,
    _ renewalDatePtr: UnsafePointer<AVPValue>?
) {
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.renewalDate = avpDate(fromISO8601: renewalDatePtr.map { String(bridge: $0) })
}

@_cdecl("av_asset_resource_loading_content_information_request_set_entire_length_available_on_demand")
public func av_asset_resource_loading_content_information_request_set_entire_length_available_on_demand(
    _ requestPtr: UnsafeMutableRawPointer,
    _ available: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("entireLengthAvailableOnDemand requires macOS 13.0")
        return AVP_INVALID_ARGUMENT
    }
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.isEntireLengthAvailableOnDemand = available
    return AVP_OK
}

@_cdecl("av_asset_resource_loading_data_request_info_value")
public func av_asset_resource_loading_data_request_info_value(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let request = Unmanaged<AVAssetResourceLoadingDataRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let payload = AssetResourceLoadingDataRequestPayload(
        requestedOffset: request.requestedOffset,
        requestedLength: Int64(request.requestedLength),
        requestsAllDataToEndOfResource: {
            if #available(macOS 10.11, *) {
                return request.requestsAllDataToEndOfResource
            }
            return nil
        }(),
        currentOffset: request.currentOffset
    )
    do {
        return try avpEncodeValue(payload)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_resource_loading_data_request_respond_with_data")
public func av_asset_resource_loading_data_request_respond_with_data(
    _ requestPtr: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<UInt8>?,
    _ count: Int
) {
    let request = Unmanaged<AVAssetResourceLoadingDataRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    let data = bytes.map { Data(bytes: $0, count: count) } ?? Data()
    request.respond(with: data)
}

@_cdecl("av_asset_resource_loading_requestor_info_value")
public func av_asset_resource_loading_requestor_info_value(
    _ requestorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutablePointer<AVPValue>? {
    let requestor = Unmanaged<AVAssetResourceLoadingRequestor>.fromOpaque(requestorPtr).takeUnretainedValue()
    do {
        return try avpEncodeValue(
            AssetResourceLoadingRequestorPayload(
                providesExpiredSessionReports: requestor.providesExpiredSessionReports
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

extension AssetResourceLoaderInfoPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            hasDelegate.binaryValue,
            preloadsEligibleContentKeys.binaryValue,
            sendsCommonMediaClientDataAsHTTPHeaders.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            hasDelegate: try binaryValue.field(0),
            preloadsEligibleContentKeys: try binaryValue.field(1),
            sendsCommonMediaClientDataAsHTTPHeaders: try binaryValue.field(2)
        )
    }
}

extension AssetResourceLoadingRequestPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            requestURL.binaryValue,
            requestMethod.binaryValue,
            requestHeaders.binaryValue,
            finished.binaryValue,
            cancelled.binaryValue,
            hasContentInformationRequest.binaryValue,
            hasDataRequest.binaryValue,
            hasRequestor.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            requestURL: try binaryValue.field(0),
            requestMethod: try binaryValue.field(1),
            requestHeaders: try binaryValue.field(2),
            finished: try binaryValue.field(3),
            cancelled: try binaryValue.field(4),
            hasContentInformationRequest: try binaryValue.field(5),
            hasDataRequest: try binaryValue.field(6),
            hasRequestor: try binaryValue.field(7)
        )
    }
}

extension AssetResourceLoadingContentInformationRequestPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            contentType.binaryValue,
            allowedContentTypes.binaryValue,
            contentLength.binaryValue,
            byteRangeAccessSupported.binaryValue,
            renewalDateISO8601.binaryValue,
            entireLengthAvailableOnDemand.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            contentType: try binaryValue.field(0),
            allowedContentTypes: try binaryValue.field(1),
            contentLength: try binaryValue.field(2),
            byteRangeAccessSupported: try binaryValue.field(3),
            renewalDateISO8601: try binaryValue.field(4),
            entireLengthAvailableOnDemand: try binaryValue.field(5)
        )
    }
}

extension AssetResourceLoadingDataRequestPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            requestedOffset.binaryValue,
            requestedLength.binaryValue,
            requestsAllDataToEndOfResource.binaryValue,
            currentOffset.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            requestedOffset: try binaryValue.field(0),
            requestedLength: try binaryValue.field(1),
            requestsAllDataToEndOfResource: try binaryValue.field(2),
            currentOffset: try binaryValue.field(3)
        )
    }
}

extension AssetResourceLoadingRequestorPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            providesExpiredSessionReports.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            providesExpiredSessionReports: try binaryValue.field(0)
        )
    }
}
