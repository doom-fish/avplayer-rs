import AVFoundation
import Foundation

private struct AssetResourceLoaderInfoPayload: Codable {
    let hasDelegate: Bool
    let preloadsEligibleContentKeys: Bool
    let sendsCommonMediaClientDataAsHTTPHeaders: Bool?
}

private struct AssetResourceLoadingRequestPayload: Codable {
    let requestURL: String?
    let requestMethod: String?
    let requestHeaders: [String: String]?
    let finished: Bool
    let cancelled: Bool
    let hasContentInformationRequest: Bool
    let hasDataRequest: Bool
    let hasRequestor: Bool
}

private struct AssetResourceLoadingContentInformationRequestPayload: Codable {
    let contentType: String?
    let allowedContentTypes: [String]?
    let contentLength: Int64
    let byteRangeAccessSupported: Bool
    let renewalDateISO8601: String?
    let entireLengthAvailableOnDemand: Bool?
}

private struct AssetResourceLoadingDataRequestPayload: Codable {
    let requestedOffset: Int64
    let requestedLength: Int64
    let requestsAllDataToEndOfResource: Bool?
    let currentOffset: Int64
}

private struct AssetResourceLoadingRequestorPayload: Codable {
    let providesExpiredSessionReports: Bool
}

private final class AssetResourceLoaderDelegateBox: NSObject, AVAssetResourceLoaderDelegate {
    private weak var loader: AVAssetResourceLoader?
    private let callback: AVPBoolObjectCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        loader: AVAssetResourceLoader,
        queue: DispatchQueue?,
        callback: @escaping AVPBoolObjectCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.loader = loader
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        loader.setDelegate(self, queue: queue)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        loader?.setDelegate(nil, queue: nil)
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    private func send(event: String, object: AnyObject) -> Bool {
        guard !disposed else { return false }
        let objectPtr = Unmanaged.passRetained(object).toOpaque()
        return event.withCString { callback(userData, $0, objectPtr) }
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

@_cdecl("av_asset_resource_loader_info_json")
public func av_asset_resource_loader_info_json(
    _ loaderPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
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
        return ffiString(try avpEncodeJSON(payload))
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
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
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
    _ queueLabel: UnsafePointer<CChar>?,
    _ callback: AVPBoolObjectCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing resource-loader callback")
        return nil
    }
    let loader = Unmanaged<AVAssetResourceLoader>.fromOpaque(loaderPtr).takeUnretainedValue()
    let observer = AssetResourceLoaderDelegateBox(
        loader: loader,
        queue: avpDispatchQueue(from: queueLabel),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_asset_resource_loader_delegate_release")
public func av_asset_resource_loader_delegate_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<AssetResourceLoaderDelegateBox>.fromOpaque(observerPtr).release()
}

@_cdecl("av_asset_resource_loading_request_info_json")
public func av_asset_resource_loading_request_info_json(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
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
        return ffiString(try avpEncodeJSON(payload))
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
    _ messagePtr: UnsafePointer<CChar>?
) {
    let request = Unmanaged<AVAssetResourceLoadingRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    if let messagePtr {
        let message = String(cString: messagePtr)
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

@_cdecl("av_asset_resource_loading_content_information_request_info_json")
public func av_asset_resource_loading_content_information_request_info_json(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
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
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_resource_loading_content_information_request_set_content_type")
public func av_asset_resource_loading_content_information_request_set_content_type(
    _ requestPtr: UnsafeMutableRawPointer,
    _ contentTypePtr: UnsafePointer<CChar>?
) {
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.contentType = contentTypePtr.map { String(cString: $0) }
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
    _ renewalDatePtr: UnsafePointer<CChar>?
) {
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.renewalDate = avpDate(fromISO8601: renewalDatePtr.map { String(cString: $0) })
}

@_cdecl("av_asset_resource_loading_content_information_request_set_entire_length_available_on_demand")
public func av_asset_resource_loading_content_information_request_set_entire_length_available_on_demand(
    _ requestPtr: UnsafeMutableRawPointer,
    _ available: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("entireLengthAvailableOnDemand requires macOS 13.0")
        return AVP_INVALID_ARGUMENT
    }
    let request = Unmanaged<AVAssetResourceLoadingContentInformationRequest>.fromOpaque(requestPtr).takeUnretainedValue()
    request.isEntireLengthAvailableOnDemand = available
    return AVP_OK
}

@_cdecl("av_asset_resource_loading_data_request_info_json")
public func av_asset_resource_loading_data_request_info_json(
    _ requestPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
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
        return ffiString(try avpEncodeJSON(payload))
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

@_cdecl("av_asset_resource_loading_requestor_info_json")
public func av_asset_resource_loading_requestor_info_json(
    _ requestorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let requestor = Unmanaged<AVAssetResourceLoadingRequestor>.fromOpaque(requestorPtr).takeUnretainedValue()
    do {
        return ffiString(
            try avpEncodeJSON(
                AssetResourceLoadingRequestorPayload(
                    providesExpiredSessionReports: requestor.providesExpiredSessionReports
                )
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
