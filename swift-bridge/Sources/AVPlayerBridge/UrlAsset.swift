import AVPlayerABI
import AVFoundation
import Foundation

@_cdecl("av_url_asset_create")
public func av_url_asset_create(
    _ urlPtr: UnsafePointer<AVPValue>,
    _ isFileURL: Bool,
    _ preferPreciseDuration: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let urlString = String(bridge: urlPtr)
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

private struct UrlAssetHTTPCookiePayload: AVPBridge {
    let name: String
    let value: String
    let domain: String
    let path: String
    let secure: Bool?
}

private struct UrlAssetOptionsPayload: AVPBridge {
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
        guard #available(macOS 13.0, *) else {
            throw BridgeError.message("HTTP user agent requires macOS 13.0 or later")
        }
        options[AVURLAssetHTTPUserAgentKey] = value
    }
    if let value = payload.primarySessionIdentifier {
        guard #available(macOS 13.0, *) else {
            throw BridgeError.message("primary session identifier requires macOS 13.0 or later")
        }
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

@_cdecl("av_url_asset_create_with_options_value")
public func av_url_asset_create_with_options_value(
    _ urlPtr: UnsafePointer<AVPValue>,
    _ isFileURL: Bool,
    _ optionsValue: UnsafePointer<AVPValue>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<AVPValue>?>?
) -> UnsafeMutableRawPointer? {
    let urlString = String(bridge: urlPtr)
    let url = isFileURL ? URL(fileURLWithPath: urlString) : URL(string: urlString)
    guard let url else {
        outErrorMessage?.pointee = ffiString("invalid URL: \(urlString)")
        return nil
    }
    do {
        let options = try avpURLAssetOptions(from: avpDecodeValue(optionsValue, as: UrlAssetOptionsPayload.self))
        return Unmanaged.passRetained(AVURLAsset(url: url, options: options)).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

extension UrlAssetHTTPCookiePayload {
    var binaryValue: AVPBinaryValue {
        .array([
            name.binaryValue,
            value.binaryValue,
            domain.binaryValue,
            path.binaryValue,
            secure.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            name: try binaryValue.field(0),
            value: try binaryValue.field(1),
            domain: try binaryValue.field(2),
            path: try binaryValue.field(3),
            secure: try binaryValue.field(4)
        )
    }
}

extension UrlAssetOptionsPayload {
    var binaryValue: AVPBinaryValue {
        .array([
            preferPreciseDurationAndTiming.binaryValue,
            overrideMimeType.binaryValue,
            referenceRestrictions.binaryValue,
            httpCookies.binaryValue,
            allowsCellularAccess.binaryValue,
            allowsExpensiveNetworkAccess.binaryValue,
            allowsConstrainedNetworkAccess.binaryValue,
            shouldSupportAliasDataReferences.binaryValue,
            urlRequestAttribution.binaryValue,
            httpUserAgent.binaryValue,
            primarySessionIdentifier.binaryValue,
            shouldParseExternalSphericalTags.binaryValue,
        ])
    }

    init(binaryValue: AVPBinaryValue) throws {
        self.init(
            preferPreciseDurationAndTiming: try binaryValue.field(0),
            overrideMimeType: try binaryValue.field(1),
            referenceRestrictions: try binaryValue.field(2),
            httpCookies: try binaryValue.field(3),
            allowsCellularAccess: try binaryValue.field(4),
            allowsExpensiveNetworkAccess: try binaryValue.field(5),
            allowsConstrainedNetworkAccess: try binaryValue.field(6),
            shouldSupportAliasDataReferences: try binaryValue.field(7),
            urlRequestAttribution: try binaryValue.field(8),
            httpUserAgent: try binaryValue.field(9),
            primarySessionIdentifier: try binaryValue.field(10),
            shouldParseExternalSphericalTags: try binaryValue.field(11)
        )
    }
}
