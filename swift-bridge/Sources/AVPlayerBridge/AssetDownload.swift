import AVFoundation
import Foundation

private struct AssetDownloadStorageManagementPolicyPayload: Codable {
    let priority: String
    let expirationDateISO8601: String?
}

private struct AssetDownloadConfigurationPayload: Codable {
    let optimizesAuxiliaryContentConfigurations: Bool
    let downloadsInterstitialAssets: Bool?
}

private struct AssetDownloadContentConfigurationPayload: Codable {
    let variantQualifierCount: Int
    let mediaSelectionCount: Int
}

@_cdecl("av_asset_download_storage_manager_shared")
public func av_asset_download_storage_manager_shared() -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.15, *) else { return nil }
    return Unmanaged.passRetained(AVAssetDownloadStorageManager.shared()).toOpaque()
}

@_cdecl("av_asset_download_storage_manager_set_policy_for_file_path")
public func av_asset_download_storage_manager_set_policy_for_file_path(
    _ managerPtr: UnsafeMutableRawPointer,
    _ policyPtr: UnsafeMutableRawPointer,
    _ pathPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadStorageManager requires macOS 10.15")
        return AVP_INVALID_ARGUMENT
    }
    let manager = Unmanaged<AVAssetDownloadStorageManager>.fromOpaque(managerPtr).takeUnretainedValue()
    let policy = Unmanaged<AVAssetDownloadStorageManagementPolicy>.fromOpaque(policyPtr).takeUnretainedValue()
    let path = String(cString: pathPtr)
    manager.setStorageManagementPolicy(policy, for: URL(fileURLWithPath: path))
    return AVP_OK
}

@_cdecl("av_asset_download_storage_manager_copy_policy_for_file_path")
public func av_asset_download_storage_manager_copy_policy_for_file_path(
    _ managerPtr: UnsafeMutableRawPointer,
    _ pathPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadStorageManager requires macOS 10.15")
        return nil
    }
    let manager = Unmanaged<AVAssetDownloadStorageManager>.fromOpaque(managerPtr).takeUnretainedValue()
    let path = String(cString: pathPtr)
    guard let policy = manager.storageManagementPolicy(for: URL(fileURLWithPath: path)) else {
        return nil
    }
    return Unmanaged.passRetained(policy).toOpaque()
}

@_cdecl("av_asset_download_storage_management_policy_create_mutable")
public func av_asset_download_storage_management_policy_create_mutable() -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.15, *) else { return nil }
    return Unmanaged.passRetained(AVMutableAssetDownloadStorageManagementPolicy()).toOpaque()
}

@_cdecl("av_asset_download_storage_management_policy_info_json")
public func av_asset_download_storage_management_policy_info_json(
    _ policyPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadStorageManagementPolicy requires macOS 10.15")
        return nil
    }
    let policy = Unmanaged<AVAssetDownloadStorageManagementPolicy>.fromOpaque(policyPtr).takeUnretainedValue()
    do {
        return ffiString(
            try avpEncodeJSON(
                AssetDownloadStorageManagementPolicyPayload(
                    priority: policy.priority.rawValue,
                    expirationDateISO8601: avpISO8601String(policy.expirationDate)
                )
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_download_storage_management_policy_set_priority")
public func av_asset_download_storage_management_policy_set_priority(
    _ policyPtr: UnsafeMutableRawPointer,
    _ priorityPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadStorageManagementPolicy requires macOS 10.15")
        return AVP_INVALID_ARGUMENT
    }
    guard let mutablePolicy = Unmanaged<AVAssetDownloadStorageManagementPolicy>.fromOpaque(policyPtr).takeUnretainedValue()
        as? AVMutableAssetDownloadStorageManagementPolicy
    else {
        outErrorMessage?.pointee = ffiString("storage management policy is not mutable")
        return AVP_INVALID_ARGUMENT
    }
    mutablePolicy.priority = AVAssetDownloadedAssetEvictionPriority(rawValue: String(cString: priorityPtr))
    return AVP_OK
}

@_cdecl("av_asset_download_storage_management_policy_set_expiration_date_iso8601")
public func av_asset_download_storage_management_policy_set_expiration_date_iso8601(
    _ policyPtr: UnsafeMutableRawPointer,
    _ valuePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadStorageManagementPolicy requires macOS 10.15")
        return AVP_INVALID_ARGUMENT
    }
    guard let mutablePolicy = Unmanaged<AVAssetDownloadStorageManagementPolicy>.fromOpaque(policyPtr).takeUnretainedValue()
        as? AVMutableAssetDownloadStorageManagementPolicy
    else {
        outErrorMessage?.pointee = ffiString("storage management policy is not mutable")
        return AVP_INVALID_ARGUMENT
    }
    guard let date = avpDate(fromISO8601: String(cString: valuePtr)) else {
        outErrorMessage?.pointee = ffiString("invalid ISO-8601 expiration date")
        return AVP_INVALID_ARGUMENT
    }
    mutablePolicy.expirationDate = date
    return AVP_OK
}

@_cdecl("av_asset_download_configuration_create")
public func av_asset_download_configuration_create(
    _ assetPtr: UnsafeMutableRawPointer,
    _ titlePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadConfiguration requires macOS 12.0")
        return nil
    }
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard let urlAsset = asset as? AVURLAsset else {
        outErrorMessage?.pointee = ffiString("download configuration requires AVURLAsset")
        return nil
    }
    let title = String(cString: titlePtr)
    return Unmanaged.passRetained(AVAssetDownloadConfiguration(asset: urlAsset, title: title)).toOpaque()
}

@_cdecl("av_asset_download_configuration_info_json")
public func av_asset_download_configuration_info_json(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadConfiguration requires macOS 12.0")
        return nil
    }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    let payload = AssetDownloadConfigurationPayload(
        optimizesAuxiliaryContentConfigurations: configuration.optimizesAuxiliaryContentConfigurations,
        downloadsInterstitialAssets: {
            if #available(macOS 15.0, *) {
                return configuration.downloadsInterstitialAssets
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

@_cdecl("av_asset_download_configuration_set_optimizes_auxiliary_content_configurations")
public func av_asset_download_configuration_set_optimizes_auxiliary_content_configurations(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    guard #available(macOS 12.0, *) else { return }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    configuration.optimizesAuxiliaryContentConfigurations = enabled
}

@_cdecl("av_asset_download_configuration_set_downloads_interstitial_assets")
public func av_asset_download_configuration_set_downloads_interstitial_assets(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ enabled: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("downloadsInterstitialAssets requires macOS 15.0")
        return AVP_INVALID_ARGUMENT
    }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    configuration.downloadsInterstitialAssets = enabled
    return AVP_OK
}

@_cdecl("av_asset_download_configuration_set_artwork_data")
public func av_asset_download_configuration_set_artwork_data(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<UInt8>?,
    _ count: Int
) {
    guard #available(macOS 12.0, *) else { return }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    configuration.artworkData = bytes.map { Data(bytes: $0, count: count) }
}

@_cdecl("av_asset_download_configuration_copy_primary_content_configuration")
public func av_asset_download_configuration_copy_primary_content_configuration(
    _ configurationPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    return Unmanaged.passRetained(configuration.primaryContentConfiguration).toOpaque()
}

@_cdecl("av_asset_download_configuration_auxiliary_content_configuration_count")
public func av_asset_download_configuration_auxiliary_content_configuration_count(
    _ configurationPtr: UnsafeMutableRawPointer
) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    return Int32(configuration.auxiliaryContentConfigurations.count)
}

@_cdecl("av_asset_download_configuration_copy_auxiliary_content_configuration_at_index")
public func av_asset_download_configuration_copy_auxiliary_content_configuration_at_index(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    let values = configuration.auxiliaryContentConfigurations
    guard index >= 0, Int(index) < values.count else { return nil }
    return Unmanaged.passRetained(values[Int(index)]).toOpaque()
}

@_cdecl("av_asset_download_configuration_set_auxiliary_content_configurations")
public func av_asset_download_configuration_set_auxiliary_content_configurations(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ contentConfigurationPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int
) {
    guard #available(macOS 12.0, *) else { return }
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    configuration.auxiliaryContentConfigurations = assetDownloadContentConfigurationsFromPointers(
        contentConfigurationPtrs,
        count: count
    )
}

@_cdecl("av_asset_download_content_configuration_create")
public func av_asset_download_content_configuration_create() -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    return Unmanaged.passRetained(AVAssetDownloadContentConfiguration()).toOpaque()
}

@_cdecl("av_asset_download_content_configuration_info_json")
public func av_asset_download_content_configuration_info_json(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadContentConfiguration requires macOS 12.0")
        return nil
    }
    let configuration = Unmanaged<AVAssetDownloadContentConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    let payload = AssetDownloadContentConfigurationPayload(
        variantQualifierCount: configuration.variantQualifiers.count,
        mediaSelectionCount: configuration.mediaSelections.count
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_download_content_configuration_media_selection_count")
public func av_asset_download_content_configuration_media_selection_count(
    _ configurationPtr: UnsafeMutableRawPointer
) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    let configuration = Unmanaged<AVAssetDownloadContentConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    return Int32(configuration.mediaSelections.count)
}

@_cdecl("av_asset_download_content_configuration_copy_media_selection_at_index")
public func av_asset_download_content_configuration_copy_media_selection_at_index(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ index: Int32
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    let configuration = Unmanaged<AVAssetDownloadContentConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    let mediaSelections = configuration.mediaSelections
    guard index >= 0, Int(index) < mediaSelections.count else { return nil }
    return Unmanaged.passRetained(mediaSelections[Int(index)]).toOpaque()
}

@_cdecl("av_asset_download_content_configuration_set_media_selections")
public func av_asset_download_content_configuration_set_media_selections(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ mediaSelectionPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int
) {
    guard #available(macOS 12.0, *) else { return }
    let configuration = Unmanaged<AVAssetDownloadContentConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    configuration.mediaSelections = mediaSelectionsFromPointers(mediaSelectionPtrs, count: count)
}

@_cdecl("av_asset_download_content_configuration_set_variant_qualifiers")
public func av_asset_download_content_configuration_set_variant_qualifiers(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ variantQualifierPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int
) {
    guard #available(macOS 12.0, *) else { return }
    let configuration = Unmanaged<AVAssetDownloadContentConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    configuration.variantQualifiers = assetVariantQualifiersFromPointers(variantQualifierPtrs, count: count)
}

@available(macOS 12.0, *)
private func assetDownloadContentConfigurationsFromPointers(
    _ pointers: UnsafePointer<UnsafeMutableRawPointer?>?,
    count: Int
) -> [AVAssetDownloadContentConfiguration] {
    guard count > 0 else { return [] }
    guard let pointers else { return [] }
    return (0..<count).compactMap {
        pointers[$0].map { Unmanaged<AVAssetDownloadContentConfiguration>.fromOpaque($0).takeUnretainedValue() }
    }
}

@available(macOS 12.0, *)
private func mediaSelectionsFromPointers(
    _ pointers: UnsafePointer<UnsafeMutableRawPointer?>?,
    count: Int
) -> [AVMediaSelection] {
    guard count > 0 else { return [] }
    guard let pointers else { return [] }
    return (0..<count).compactMap {
        pointers[$0].map { Unmanaged<AVMediaSelection>.fromOpaque($0).takeUnretainedValue() }
    }
}

@available(macOS 12.0, *)
private func assetVariantQualifiersFromPointers(
    _ pointers: UnsafePointer<UnsafeMutableRawPointer?>?,
    count: Int
) -> [AVAssetVariantQualifier] {
    guard count > 0 else { return [] }
    guard let pointers else { return [] }
    return (0..<count).compactMap {
        pointers[$0].map { Unmanaged<AVAssetVariantQualifier>.fromOpaque($0).takeUnretainedValue() }
    }
}

private struct AssetDownloadTaskInfoPayload: Codable {
    let taskIdentifier: Int
    let urlAssetURL: String?
    let state: Int
    let loadedTimeRanges: [TimeRangePayload]
}

private struct AssetDownloadDelegateEventPayload: Codable {
    let event: String
    let taskIdentifier: Int
    let location: String?
    let loadedTimeRange: TimeRangePayload?
    let totalTimeRangesLoaded: [TimeRangePayload]?
    let timeRangeExpectedToLoad: TimeRangePayload?
    let variantCount: Int?
    let metricEventClassName: String?
}

private final class AssetDownloadDelegateBridge: NSObject, AVAssetDownloadDelegate {
    private let callback: AVPJsonCallback?
    private let userDataBits: UInt
    private let hasUserData: Bool
    private let dropUserData: AVPDropCallback?
    private var disposed = false

    init(
        callback: AVPJsonCallback?,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.callback = callback
        userDataBits = userData.map { UInt(bitPattern: $0) } ?? 0
        hasUserData = userData != nil
        self.dropUserData = dropUserData
        super.init()
    }

    deinit {
        dispose()
    }

    private var userDataPointer: UnsafeMutableRawPointer? {
        guard hasUserData else { return nil }
        return UnsafeMutableRawPointer(bitPattern: userDataBits)
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        if let userData = userDataPointer, let dropUserData {
            dropUserData(userData)
        }
    }

    private func send(_ payload: AssetDownloadDelegateEventPayload) {
        guard !disposed, let callback else { return }
        guard let json = try? avpEncodeJSON(payload) else { return }
        json.withCString { callback(userDataPointer, $0) }
    }

    func urlSession(
        _ session: URLSession,
        assetDownloadTask: AVAssetDownloadTask,
        didLoad timeRange: CMTimeRange,
        totalTimeRangesLoaded loadedTimeRanges: [NSValue],
        timeRangeExpectedToLoad: CMTimeRange
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "did_load_time_range",
                taskIdentifier: assetDownloadTask.taskIdentifier,
                location: nil,
                loadedTimeRange: encodeTimeRange(timeRange),
                totalTimeRangesLoaded: loadedTimeRanges.map { encodeTimeRange($0.timeRangeValue) },
                timeRangeExpectedToLoad: encodeTimeRange(timeRangeExpectedToLoad),
                variantCount: nil,
                metricEventClassName: nil
            )
        )
    }

    func urlSession(
        _ session: URLSession,
        assetDownloadTask: AVAssetDownloadTask,
        didResolve resolvedMediaSelection: AVMediaSelection
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "did_resolve_media_selection",
                taskIdentifier: assetDownloadTask.taskIdentifier,
                location: nil,
                loadedTimeRange: nil,
                totalTimeRangesLoaded: nil,
                timeRangeExpectedToLoad: nil,
                variantCount: nil,
                metricEventClassName: nil
            )
        )
    }

    func urlSession(
        _ session: URLSession,
        assetDownloadTask: AVAssetDownloadTask,
        willDownloadTo location: URL
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "will_download_to_url",
                taskIdentifier: assetDownloadTask.taskIdentifier,
                location: location.absoluteString,
                loadedTimeRange: nil,
                totalTimeRangesLoaded: nil,
                timeRangeExpectedToLoad: nil,
                variantCount: nil,
                metricEventClassName: nil
            )
        )
    }

    func urlSession(
        _ session: URLSession,
        aggregateAssetDownloadTask: AVAggregateAssetDownloadTask,
        willDownloadTo location: URL
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "aggregate_will_download_to_url",
                taskIdentifier: aggregateAssetDownloadTask.taskIdentifier,
                location: location.absoluteString,
                loadedTimeRange: nil,
                totalTimeRangesLoaded: nil,
                timeRangeExpectedToLoad: nil,
                variantCount: nil,
                metricEventClassName: nil
            )
        )
    }

    func urlSession(
        _ session: URLSession,
        aggregateAssetDownloadTask: AVAggregateAssetDownloadTask,
        didCompleteFor mediaSelection: AVMediaSelection
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "aggregate_did_complete_for_media_selection",
                taskIdentifier: aggregateAssetDownloadTask.taskIdentifier,
                location: nil,
                loadedTimeRange: nil,
                totalTimeRangesLoaded: nil,
                timeRangeExpectedToLoad: nil,
                variantCount: nil,
                metricEventClassName: nil
            )
        )
    }

    func urlSession(
        _ session: URLSession,
        aggregateAssetDownloadTask: AVAggregateAssetDownloadTask,
        didLoad timeRange: CMTimeRange,
        totalTimeRangesLoaded loadedTimeRanges: [NSValue],
        timeRangeExpectedToLoad: CMTimeRange,
        for mediaSelection: AVMediaSelection
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "aggregate_did_load_time_range",
                taskIdentifier: aggregateAssetDownloadTask.taskIdentifier,
                location: nil,
                loadedTimeRange: encodeTimeRange(timeRange),
                totalTimeRangesLoaded: loadedTimeRanges.map { encodeTimeRange($0.timeRangeValue) },
                timeRangeExpectedToLoad: encodeTimeRange(timeRangeExpectedToLoad),
                variantCount: nil,
                metricEventClassName: nil
            )
        )
    }

    func urlSession(
        _ session: URLSession,
        assetDownloadTask: AVAssetDownloadTask,
        willDownloadVariants variants: [AVAssetVariant]
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "will_download_variants",
                taskIdentifier: assetDownloadTask.taskIdentifier,
                location: nil,
                loadedTimeRange: nil,
                totalTimeRangesLoaded: nil,
                timeRangeExpectedToLoad: nil,
                variantCount: variants.count,
                metricEventClassName: nil
            )
        )
    }

    @available(macOS 26.0, *)
    func urlSession(
        _ session: URLSession,
        assetDownloadTask: AVAssetDownloadTask,
        didReceive metricEvent: AVMetricEvent
    ) {
        send(
            AssetDownloadDelegateEventPayload(
                event: "did_receive_metric_event",
                taskIdentifier: assetDownloadTask.taskIdentifier,
                location: nil,
                loadedTimeRange: nil,
                totalTimeRangesLoaded: nil,
                timeRangeExpectedToLoad: nil,
                variantCount: nil,
                metricEventClassName: String(describing: type(of: metricEvent))
            )
        )
    }
}

private final class AssetDownloadURLSessionBox: NSObject {
    let delegateBridge: AssetDownloadDelegateBridge
    let delegateQueue: OperationQueue?
    let underlyingQueue: DispatchQueue?
    let session: AVAssetDownloadURLSession

    init(identifier: String, queueLabel: String?, callback: AVPJsonCallback?, userData: UnsafeMutableRawPointer?, dropUserData: AVPDropCallback?) {
        delegateBridge = AssetDownloadDelegateBridge(
            callback: callback,
            userData: userData,
            dropUserData: dropUserData
        )
        if let queueLabel {
            let queue = OperationQueue()
            queue.name = queueLabel
            queue.maxConcurrentOperationCount = 1
            queue.qualityOfService = .utility
            let dispatchQueue = DispatchQueue(label: queueLabel)
            queue.underlyingQueue = dispatchQueue
            delegateQueue = queue
            underlyingQueue = dispatchQueue
        } else {
            delegateQueue = nil
            underlyingQueue = nil
        }
        let configuration = URLSessionConfiguration.background(withIdentifier: identifier)
        session = AVAssetDownloadURLSession(
            configuration: configuration,
            assetDownloadDelegate: delegateBridge,
            delegateQueue: delegateQueue
        )
        super.init()
    }

    deinit {
        delegateBridge.dispose()
    }
}

private func makeAssetDownloadTaskInfoPayload(_ task: AVAssetDownloadTask) -> AssetDownloadTaskInfoPayload {
    AssetDownloadTaskInfoPayload(
        taskIdentifier: task.taskIdentifier,
        urlAssetURL: task.urlAsset.url.absoluteString,
        state: task.state.rawValue,
        loadedTimeRanges: task.loadedTimeRanges.map { encodeTimeRange($0.timeRangeValue) }
    )
}

private func makeAggregateAssetDownloadTaskInfoPayload(_ task: AVAggregateAssetDownloadTask) -> AssetDownloadTaskInfoPayload {
    AssetDownloadTaskInfoPayload(
        taskIdentifier: task.taskIdentifier,
        urlAssetURL: task.urlAsset.url.absoluteString,
        state: task.state.rawValue,
        loadedTimeRanges: []
    )
}

@_cdecl("av_asset_download_url_session_create_background")
public func av_asset_download_url_session_create_background(
    _ identifierPtr: UnsafePointer<CChar>,
    _ queueLabelPtr: UnsafePointer<CChar>?,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadURLSession requires macOS 10.15")
        return nil
    }
    let identifier = String(cString: identifierPtr)
    let queueLabel = queueLabelPtr.map { String(cString: $0) }
    let box = AssetDownloadURLSessionBox(
        identifier: identifier,
        queueLabel: queueLabel,
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(box).toOpaque()
}

@_cdecl("av_asset_download_url_session_release")
public func av_asset_download_url_session_release(_ sessionPtr: UnsafeMutableRawPointer?) {
    guard let sessionPtr else { return }
    Unmanaged<AssetDownloadURLSessionBox>.fromOpaque(sessionPtr).release()
}

@_cdecl("av_asset_download_url_session_finish_tasks_and_invalidate")
public func av_asset_download_url_session_finish_tasks_and_invalidate(
    _ sessionPtr: UnsafeMutableRawPointer
) {
    let box = Unmanaged<AssetDownloadURLSessionBox>.fromOpaque(sessionPtr).takeUnretainedValue()
    box.session.finishTasksAndInvalidate()
}

@_cdecl("av_asset_download_url_session_invalidate_and_cancel")
public func av_asset_download_url_session_invalidate_and_cancel(
    _ sessionPtr: UnsafeMutableRawPointer
) {
    let box = Unmanaged<AssetDownloadURLSessionBox>.fromOpaque(sessionPtr).takeUnretainedValue()
    box.session.invalidateAndCancel()
}

@_cdecl("av_asset_download_url_session_create_task_with_configuration")
public func av_asset_download_url_session_create_task_with_configuration(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ configurationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("AVAssetDownloadTask requires macOS 12.0")
        return nil
    }
    let box = Unmanaged<AssetDownloadURLSessionBox>.fromOpaque(sessionPtr).takeUnretainedValue()
    let configuration = Unmanaged<AVAssetDownloadConfiguration>.fromOpaque(configurationPtr).takeUnretainedValue()
    return Unmanaged.passRetained(box.session.makeAssetDownloadTask(downloadConfiguration: configuration)).toOpaque()
}

@_cdecl("av_asset_download_url_session_create_aggregate_task")
public func av_asset_download_url_session_create_aggregate_task(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ assetPtr: UnsafeMutableRawPointer,
    _ mediaSelectionPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int,
    _ titlePtr: UnsafePointer<CChar>,
    _ artworkBytes: UnsafePointer<UInt8>?,
    _ artworkLen: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("AVAggregateAssetDownloadTask requires macOS 10.15")
        return nil
    }
    let box = Unmanaged<AssetDownloadURLSessionBox>.fromOpaque(sessionPtr).takeUnretainedValue()
    let asset = Unmanaged<AVAsset>.fromOpaque(assetPtr).takeUnretainedValue()
    guard let urlAsset = asset as? AVURLAsset else {
        outErrorMessage?.pointee = ffiString("aggregate asset download requires AVURLAsset")
        return nil
    }
    let task = box.session.aggregateAssetDownloadTask(
        with: urlAsset,
        mediaSelections: mediaSelectionsFromPointers(mediaSelectionPtrs, count: count),
        assetTitle: String(cString: titlePtr),
        assetArtworkData: artworkBytes.map { Data(bytes: $0, count: artworkLen) },
        options: nil
    )
    guard let task else {
        outErrorMessage?.pointee = ffiString("asset download URLSession returned nil aggregate task")
        return nil
    }
    return Unmanaged.passRetained(task).toOpaque()
}

@_cdecl("av_asset_download_task_info_json")
public func av_asset_download_task_info_json(
    _ taskPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let task = Unmanaged<AVAssetDownloadTask>.fromOpaque(taskPtr).takeUnretainedValue()
    do {
        return ffiString(try avpEncodeJSON(makeAssetDownloadTaskInfoPayload(task)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_aggregate_asset_download_task_info_json")
public func av_aggregate_asset_download_task_info_json(
    _ taskPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let task = Unmanaged<AVAggregateAssetDownloadTask>.fromOpaque(taskPtr).takeUnretainedValue()
    do {
        return ffiString(try avpEncodeJSON(makeAggregateAssetDownloadTaskInfoPayload(task)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_asset_download_task_resume")
public func av_asset_download_task_resume(_ taskPtr: UnsafeMutableRawPointer) {
    let task = Unmanaged<URLSessionTask>.fromOpaque(taskPtr).takeUnretainedValue()
    task.resume()
}

@_cdecl("av_asset_download_task_suspend")
public func av_asset_download_task_suspend(_ taskPtr: UnsafeMutableRawPointer) {
    let task = Unmanaged<URLSessionTask>.fromOpaque(taskPtr).takeUnretainedValue()
    task.suspend()
}

@_cdecl("av_asset_download_task_cancel")
public func av_asset_download_task_cancel(_ taskPtr: UnsafeMutableRawPointer) {
    let task = Unmanaged<URLSessionTask>.fromOpaque(taskPtr).takeUnretainedValue()
    task.cancel()
}
