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

private struct UrlAssetHTTPCookiePayload: Codable {
    let name: String
    let value: String
    let domain: String
    let path: String
    let secure: Bool?
}

private struct UrlAssetOptionsPayload: Codable {
    let preferPreciseDurationAndTiming: Bool?
    let overrideMimeType: String?
    let referenceRestrictions: UInt?
    let httpCookies: [UrlAssetHTTPCookiePayload]?
    let allowsCellularAccess: Bool?
    let allowsExpensiveNetworkAccess: Bool?
    let allowsConstrainedNetworkAccess: Bool?
    let shouldSupportAliasDataReferences: Bool?
    let urlRequestAttribution: UInt?
    let httpUserAgent: String?
    let primarySessionIdentifier: String?
    let shouldParseExternalSphericalTags: Bool?
}

private func avpURLAssetOptions(from payload: UrlAssetOptionsPayload) throws -> [String: Any] {
    var options: [String: Any] = [:]
    if let value = payload.preferPreciseDurationAndTiming {
        options[AVURLAssetPreferPreciseDurationAndTimingKey] = value
    }
    if let value = payload.overrideMimeType {
        guard #available(macOS 14.0, *) else {
            throw BridgeError.message("AVURLAssetOverrideMIMETypeKey requires macOS 14.0+")
        }
        options[AVURLAssetOverrideMIMETypeKey] = value
    }
    if let value = payload.referenceRestrictions {
        options[AVURLAssetReferenceRestrictionsKey] = value
    }
    if let cookies = payload.httpCookies {
        options[AVURLAssetHTTPCookiesKey] = try cookies.map { cookie -> HTTPCookie in
            var properties: [HTTPCookiePropertyKey: Any] = [
                .name: cookie.name,
                .value: cookie.value,
                .domain: cookie.domain,
                .path: cookie.path,
            ]
            if cookie.secure == true {
                properties[.secure] = "TRUE"
            }
            guard let httpCookie = HTTPCookie(properties: properties) else {
                throw BridgeError.message("invalid HTTP cookie: \(cookie.name)")
            }
            return httpCookie
        }
    }
    if let value = payload.allowsCellularAccess {
        options[AVURLAssetAllowsCellularAccessKey] = value
    }
    if let value = payload.allowsExpensiveNetworkAccess {
        options[AVURLAssetAllowsExpensiveNetworkAccessKey] = value
    }
    if let value = payload.allowsConstrainedNetworkAccess {
        options[AVURLAssetAllowsConstrainedNetworkAccessKey] = value
    }
    if let value = payload.shouldSupportAliasDataReferences {
        options[AVURLAssetShouldSupportAliasDataReferencesKey] = value
    }
    if let value = payload.urlRequestAttribution {
        guard NSURLRequest.Attribution(rawValue: value) != nil else {
            throw BridgeError.message("invalid NSURLRequestAttribution raw value: \(value)")
        }
        options[AVURLAssetURLRequestAttributionKey] = value
    }
    if let value = payload.httpUserAgent {
        options[AVURLAssetHTTPUserAgentKey] = value
    }
    if let value = payload.primarySessionIdentifier {
        guard let identifier = UUID(uuidString: value) else {
            throw BridgeError.message("primary session identifier is not a UUID: \(value)")
        }
        options[AVURLAssetPrimarySessionIdentifierKey] = identifier as NSUUID
    }
    if let value = payload.shouldParseExternalSphericalTags {
        guard #available(macOS 26.0, *) else {
            throw BridgeError.message("AVURLAssetShouldParseExternalSphericalTagsKey requires macOS 26.0+")
        }
        options[AVURLAssetShouldParseExternalSphericalTagsKey] = value
    }
    return options
}

@_cdecl("av_url_asset_create_with_options_json")
public func av_url_asset_create_with_options_json(
    _ urlPtr: UnsafePointer<CChar>,
    _ isFileURL: Bool,
    _ optionsJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let urlString = String(cString: urlPtr)
    let url = isFileURL ? URL(fileURLWithPath: urlString) : URL(string: urlString)
    guard let url else {
        outErrorMessage?.pointee = ffiString("invalid URL: \(urlString)")
        return nil
    }
    do {
        let options = try avpURLAssetOptions(from: avpDecodeJSON(optionsJson, as: UrlAssetOptionsPayload.self))
        return Unmanaged.passRetained(AVURLAsset(url: url, options: options)).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
