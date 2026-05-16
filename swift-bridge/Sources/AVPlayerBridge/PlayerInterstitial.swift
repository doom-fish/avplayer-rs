import AVFoundation
import Foundation

struct PlayerInterstitialEventInfoPayload: Codable {
    let identifier: String
    let time: TimePayload
    let date: String?
    let templateItemCount: Int
    let restrictions: UInt
    let resumptionOffset: TimePayload
    let playoutLimit: TimePayload
    let alignsStartWithPrimarySegmentBoundary: Bool
    let alignsResumptionWithPrimarySegmentBoundary: Bool
    let cue: String?
    let willPlayOnce: Bool
    let userDefinedAttributesJSON: String?
    let assetListResponseJSON: String?
    let timelineOccupancyRaw: Int?
    let supplementsPrimaryContent: Bool?
    let contentMayVary: Bool?
    let hasPrimaryItem: Bool
}

struct PlayerInterstitialMonitorStatePayload: Codable {
    let events: [PlayerInterstitialEventInfoPayload]
    let currentEvent: PlayerInterstitialEventInfoPayload?
    let currentEventSkippableStateRaw: Int?
    let currentEventSkipControlLabel: String?
}

struct PlayerInterstitialMonitorEventPayload: Codable {
    let event: String
    let interstitialEvent: PlayerInterstitialEventInfoPayload?
    let assetListResponseStatusRaw: Int?
    let skippableStateRaw: Int?
    let skipControlLabel: String?
    let errorMessage: String?
    let playoutTime: TimePayload?
    let didPlayEntireEvent: Bool?
}

class AVPPlayerInterstitialEventBox: NSObject {
    let event: AVPlayerInterstitialEvent

    init(event: AVPlayerInterstitialEvent) {
        self.event = event
    }
}

class AVPPlayerInterstitialEventMonitorBox: NSObject {
    let monitor: AVPlayerInterstitialEventMonitor

    init(monitor: AVPlayerInterstitialEventMonitor) {
        self.monitor = monitor
    }
}

final class AVPPlayerInterstitialEventControllerBox: AVPPlayerInterstitialEventMonitorBox {
    let controller: AVPlayerInterstitialEventController

    init(controller: AVPlayerInterstitialEventController) {
        self.controller = controller
        super.init(monitor: controller)
    }
}

final class InterstitialEventMonitorObserverBox: NSObject {
    private weak var monitor: AVPlayerInterstitialEventMonitor?
    private let callback: AVPJsonCallback
    private let userData: UnsafeMutableRawPointer?
    private let dropUserData: AVPDropCallback?
    private var disposed = false
    private var observers: [NSObjectProtocol] = []

    init(
        monitor: AVPlayerInterstitialEventMonitor,
        callback: @escaping AVPJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVPDropCallback?
    ) {
        self.monitor = monitor
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
        super.init()
        registerObservers(for: monitor)
    }

    deinit {
        dispose()
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        observers.forEach(NotificationCenter.default.removeObserver)
        observers.removeAll()
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }

    private func registerObservers(for monitor: AVPlayerInterstitialEventMonitor) {
        let center = NotificationCenter.default
        observers.append(
            center.addObserver(
                forName: Notification.Name(rawValue: "AVPlayerInterstitialEventMonitorEventsDidChangeNotification"),
                object: monitor,
                queue: nil
            ) { [weak self] _ in
                self?.send(
                    PlayerInterstitialMonitorEventPayload(
                        event: "events_did_change",
                        interstitialEvent: nil,
                        assetListResponseStatusRaw: nil,
                        skippableStateRaw: nil,
                        skipControlLabel: nil,
                        errorMessage: nil,
                        playoutTime: nil,
                        didPlayEntireEvent: nil
                    )
                )
            }
        )
        observers.append(
            center.addObserver(
                forName: Notification.Name(rawValue: "AVPlayerInterstitialEventMonitorCurrentEventDidChangeNotification"),
                object: monitor,
                queue: nil
            ) { [weak self] _ in
                self?.send(
                    PlayerInterstitialMonitorEventPayload(
                        event: "current_event_did_change",
                        interstitialEvent: nil,
                        assetListResponseStatusRaw: nil,
                        skippableStateRaw: nil,
                        skipControlLabel: nil,
                        errorMessage: nil,
                        playoutTime: nil,
                        didPlayEntireEvent: nil
                    )
                )
            }
        )
        observers.append(
            center.addObserver(
                forName: Notification.Name(rawValue: "AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeNotification"),
                object: monitor,
                queue: nil
            ) { [weak self] notification in
                let userInfo = notification.userInfo
                self?.send(
                    PlayerInterstitialMonitorEventPayload(
                        event: "asset_list_response_status_did_change",
                        interstitialEvent: (userInfo?["AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeEventKey"] as? AVPlayerInterstitialEvent).map(encodeInterstitialEventSummary),
                        assetListResponseStatusRaw: (userInfo?["AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeStatusKey"] as? NSNumber)?.intValue,
                        skippableStateRaw: nil,
                        skipControlLabel: nil,
                        errorMessage: (userInfo?["AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeErrorKey"] as? NSError)?.localizedDescription,
                        playoutTime: nil,
                        didPlayEntireEvent: nil
                    )
                )
            }
        )
        if #available(macOS 26.0, *) {
            observers.append(
                center.addObserver(
                    forName: Notification.Name(rawValue: "AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeNotification"),
                    object: monitor,
                    queue: nil
                ) { [weak self] notification in
                    let userInfo = notification.userInfo
                    self?.send(
                        PlayerInterstitialMonitorEventPayload(
                            event: "current_event_skippable_state_did_change",
                            interstitialEvent: (userInfo?["AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeEventKey"] as? AVPlayerInterstitialEvent).map(encodeInterstitialEventSummary),
                            assetListResponseStatusRaw: nil,
                            skippableStateRaw: (userInfo?["AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeStateKey"] as? NSNumber)?.intValue,
                            skipControlLabel: userInfo?["AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeSkipControlLabelKey"] as? String,
                            errorMessage: nil,
                            playoutTime: nil,
                            didPlayEntireEvent: nil
                        )
                    )
                }
            )
            observers.append(
                center.addObserver(
                    forName: Notification.Name(rawValue: "AVPlayerInterstitialEventMonitorCurrentEventSkippedNotification"),
                    object: monitor,
                    queue: nil
                ) { [weak self] notification in
                    let userInfo = notification.userInfo
                    self?.send(
                        PlayerInterstitialMonitorEventPayload(
                            event: "current_event_skipped",
                            interstitialEvent: (userInfo?["AVPlayerInterstitialEventMonitorCurrentEventSkippedEventKey"] as? AVPlayerInterstitialEvent).map(encodeInterstitialEventSummary),
                            assetListResponseStatusRaw: nil,
                            skippableStateRaw: nil,
                            skipControlLabel: nil,
                            errorMessage: nil,
                            playoutTime: nil,
                            didPlayEntireEvent: nil
                        )
                    )
                }
            )
            observers.append(
                center.addObserver(
                    forName: Notification.Name(rawValue: "AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledNotification"),
                    object: monitor,
                    queue: nil
                ) { [weak self] notification in
                    let userInfo = notification.userInfo
                    self?.send(
                        PlayerInterstitialMonitorEventPayload(
                            event: "interstitial_event_was_unscheduled",
                            interstitialEvent: (userInfo?["AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledEventKey"] as? AVPlayerInterstitialEvent).map(encodeInterstitialEventSummary),
                            assetListResponseStatusRaw: nil,
                            skippableStateRaw: nil,
                            skipControlLabel: nil,
                            errorMessage: (userInfo?["AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledErrorKey"] as? NSError)?.localizedDescription,
                            playoutTime: nil,
                            didPlayEntireEvent: nil
                        )
                    )
                }
            )
            observers.append(
                center.addObserver(
                    forName: Notification.Name(rawValue: "AVPlayerInterstitialEventMonitorInterstitialEventDidFinishNotification"),
                    object: monitor,
                    queue: nil
                ) { [weak self] notification in
                    let userInfo = notification.userInfo
                    let playoutTime = (userInfo?["AVPlayerInterstitialEventMonitorInterstitialEventDidFinishPlayoutTimeKey"] as? NSValue).map { encodeTime($0.timeValue) }
                    self?.send(
                        PlayerInterstitialMonitorEventPayload(
                            event: "interstitial_event_did_finish",
                            interstitialEvent: (userInfo?["AVPlayerInterstitialEventMonitorInterstitialEventDidFinishEventKey"] as? AVPlayerInterstitialEvent).map(encodeInterstitialEventSummary),
                            assetListResponseStatusRaw: nil,
                            skippableStateRaw: nil,
                            skipControlLabel: nil,
                            errorMessage: nil,
                            playoutTime: playoutTime,
                            didPlayEntireEvent: (userInfo?["AVPlayerInterstitialEventMonitorInterstitialEventDidFinishDidPlayEntireEventKey"] as? NSNumber)?.boolValue
                        )
                    )
                }
            )
        }
    }

    private func send(_ payload: PlayerInterstitialMonitorEventPayload) {
        guard !disposed else { return }
        guard let json = try? avpEncodeJSON(payload) else {
            callback(userData, nil)
            return
        }
        json.withCString { callback(userData, $0) }
    }
}

@_cdecl("av_player_interstitial_event_create_with_time")
public func av_player_interstitial_event_create_with_time(
    _ itemPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let item = Unmanaged<AVPlayerItem>.fromOpaque(itemPtr).takeUnretainedValue()
    let event = AVPlayerInterstitialEvent(primaryItem: item, time: cmTime(value: value, timescale: timescale, kind: kind))
    return Unmanaged.passRetained(AVPPlayerInterstitialEventBox(event: event)).toOpaque()
}

@_cdecl("av_player_interstitial_event_release")
public func av_player_interstitial_event_release(_ eventPtr: UnsafeMutableRawPointer?) {
    guard let eventPtr else { return }
    Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).release()
}

@_cdecl("av_player_interstitial_event_info_json")
public func av_player_interstitial_event_info_json(
    _ eventPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    do {
        return ffiString(try avpEncodeJSON(encodeInterstitialEventSummary(event)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_interstitial_event_set_identifier")
public func av_player_interstitial_event_set_identifier(
    _ eventPtr: UnsafeMutableRawPointer,
    _ identifierPtr: UnsafePointer<CChar>
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.identifier = String(cString: identifierPtr)
}

@_cdecl("av_player_interstitial_event_set_restrictions")
public func av_player_interstitial_event_set_restrictions(
    _ eventPtr: UnsafeMutableRawPointer,
    _ restrictions: UInt
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.restrictions = AVPlayerInterstitialEvent.Restrictions(rawValue: restrictions)
}

@_cdecl("av_player_interstitial_event_set_resumption_offset")
public func av_player_interstitial_event_set_resumption_offset(
    _ eventPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.resumptionOffset = cmTime(value: value, timescale: timescale, kind: kind)
}

@_cdecl("av_player_interstitial_event_set_playout_limit")
public func av_player_interstitial_event_set_playout_limit(
    _ eventPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.playoutLimit = cmTime(value: value, timescale: timescale, kind: kind)
}

@_cdecl("av_player_interstitial_event_set_aligns_start_with_primary_segment_boundary")
public func av_player_interstitial_event_set_aligns_start_with_primary_segment_boundary(
    _ eventPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.alignsStartWithPrimarySegmentBoundary = enabled
}

@_cdecl("av_player_interstitial_event_set_aligns_resumption_with_primary_segment_boundary")
public func av_player_interstitial_event_set_aligns_resumption_with_primary_segment_boundary(
    _ eventPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.alignsResumptionWithPrimarySegmentBoundary = enabled
}

@_cdecl("av_player_interstitial_event_set_cue")
public func av_player_interstitial_event_set_cue(
    _ eventPtr: UnsafeMutableRawPointer,
    _ cuePtr: UnsafePointer<CChar>
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.cue = interstitialCue(from: String(cString: cuePtr))
}

@_cdecl("av_player_interstitial_event_set_will_play_once")
public func av_player_interstitial_event_set_will_play_once(
    _ eventPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.willPlayOnce = enabled
}

@_cdecl("av_player_interstitial_event_set_timeline_occupancy")
public func av_player_interstitial_event_set_timeline_occupancy(
    _ eventPtr: UnsafeMutableRawPointer,
    _ rawValue: Int32
) {
    guard #available(macOS 15.0, *) else { return }
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.timelineOccupancy = rawValue == 1 ? .fill : .singlePoint
}

@_cdecl("av_player_interstitial_event_set_supplements_primary_content")
public func av_player_interstitial_event_set_supplements_primary_content(
    _ eventPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    guard #available(macOS 15.0, *) else { return }
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.supplementsPrimaryContent = enabled
}

@_cdecl("av_player_interstitial_event_set_content_may_vary")
public func av_player_interstitial_event_set_content_may_vary(
    _ eventPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    guard #available(macOS 15.0, *) else { return }
    let event = Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    event.contentMayVary = enabled
}

@_cdecl("av_player_interstitial_event_monitor_create")
public func av_player_interstitial_event_monitor_create(
    _ playerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let monitor = AVPlayerInterstitialEventMonitor(primaryPlayer: player)
    return Unmanaged.passRetained(AVPPlayerInterstitialEventMonitorBox(monitor: monitor)).toOpaque()
}

@_cdecl("av_player_interstitial_event_monitor_release")
public func av_player_interstitial_event_monitor_release(_ monitorPtr: UnsafeMutableRawPointer?) {
    guard let monitorPtr else { return }
    Unmanaged<AVPPlayerInterstitialEventMonitorBox>.fromOpaque(monitorPtr).release()
}

@_cdecl("av_player_interstitial_event_monitor_info_json")
public func av_player_interstitial_event_monitor_info_json(
    _ monitorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let monitor = Unmanaged<AVPPlayerInterstitialEventMonitorBox>.fromOpaque(monitorPtr).takeUnretainedValue().monitor
    do {
        return ffiString(try avpEncodeJSON(encodeInterstitialMonitorState(monitor)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_interstitial_event_monitor_add_observer")
public func av_player_interstitial_event_monitor_add_observer(
    _ monitorPtr: UnsafeMutableRawPointer,
    _ callback: AVPJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVPDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing interstitial monitor observer callback")
        return nil
    }
    let monitor = Unmanaged<AVPPlayerInterstitialEventMonitorBox>.fromOpaque(monitorPtr).takeUnretainedValue().monitor
    let observer = InterstitialEventMonitorObserverBox(
        monitor: monitor,
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("av_player_interstitial_event_monitor_observer_release")
public func av_player_interstitial_event_monitor_observer_release(_ observerPtr: UnsafeMutableRawPointer?) {
    guard let observerPtr else { return }
    Unmanaged<InterstitialEventMonitorObserverBox>.fromOpaque(observerPtr).release()
}

@_cdecl("av_player_interstitial_event_controller_create")
public func av_player_interstitial_event_controller_create(
    _ playerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let player = Unmanaged<AVPlayer>.fromOpaque(playerPtr).takeUnretainedValue()
    let controller = AVPlayerInterstitialEventController(primaryPlayer: player)
    return Unmanaged.passRetained(AVPPlayerInterstitialEventControllerBox(controller: controller)).toOpaque()
}

@_cdecl("av_player_interstitial_event_controller_release")
public func av_player_interstitial_event_controller_release(_ controllerPtr: UnsafeMutableRawPointer?) {
    guard let controllerPtr else { return }
    Unmanaged<AVPPlayerInterstitialEventControllerBox>.fromOpaque(controllerPtr).release()
}

@_cdecl("av_player_interstitial_event_controller_info_json")
public func av_player_interstitial_event_controller_info_json(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let controller = Unmanaged<AVPPlayerInterstitialEventControllerBox>.fromOpaque(controllerPtr).takeUnretainedValue().controller
    do {
        return ffiString(try avpEncodeJSON(encodeInterstitialMonitorState(controller)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_interstitial_event_controller_set_events")
public func av_player_interstitial_event_controller_set_events(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ eventPtrs: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let controller = Unmanaged<AVPPlayerInterstitialEventControllerBox>.fromOpaque(controllerPtr).takeUnretainedValue().controller
    let events = (0..<count).compactMap { index -> AVPlayerInterstitialEvent? in
        guard let eventPtrs else { return nil }
        guard let eventPtr = eventPtrs[index] else { return nil }
        return Unmanaged<AVPPlayerInterstitialEventBox>.fromOpaque(eventPtr).takeUnretainedValue().event
    }
    guard events.count == count else {
        outErrorMessage?.pointee = ffiString("controller events list contained nil")
        return AVP_INVALID_ARGUMENT
    }
    controller.events = events
    return AVP_OK
}

@_cdecl("av_player_interstitial_event_controller_cancel_current_event_with_resumption_offset")
public func av_player_interstitial_event_controller_cancel_current_event_with_resumption_offset(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ kind: Int32
) {
    let controller = Unmanaged<AVPPlayerInterstitialEventControllerBox>.fromOpaque(controllerPtr).takeUnretainedValue().controller
    controller.cancelCurrentEvent(withResumptionOffset: cmTime(value: value, timescale: timescale, kind: kind))
}

@_cdecl("av_player_interstitial_event_controller_skip_current_event")
public func av_player_interstitial_event_controller_skip_current_event(_ controllerPtr: UnsafeMutableRawPointer) {
    guard #available(macOS 26.0, *) else { return }
    let controller = Unmanaged<AVPPlayerInterstitialEventControllerBox>.fromOpaque(controllerPtr).takeUnretainedValue().controller
    controller.skipCurrentEvent()
}

@_cdecl("av_player_waiting_during_interstitial_event_reason")
public func av_player_waiting_during_interstitial_event_reason(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        return ffiString(try avpEncodeJSON(AVPlayer.WaitingReason.interstitialEvent.rawValue))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

func encodeInterstitialEventSummary(_ event: AVPlayerInterstitialEvent) -> PlayerInterstitialEventInfoPayload {
    PlayerInterstitialEventInfoPayload(
        identifier: event.identifier,
        time: encodeTime(event.time),
        date: avpISO8601String(event.date),
        templateItemCount: event.templateItems.count,
        restrictions: event.restrictions.rawValue,
        resumptionOffset: encodeTime(event.resumptionOffset),
        playoutLimit: encodeTime(event.playoutLimit),
        alignsStartWithPrimarySegmentBoundary: event.alignsStartWithPrimarySegmentBoundary,
        alignsResumptionWithPrimarySegmentBoundary: event.alignsResumptionWithPrimarySegmentBoundary,
        cue: interstitialCueString(event.cue),
        willPlayOnce: event.willPlayOnce,
        userDefinedAttributesJSON: jsonString(fromJSONObject: event.userDefinedAttributes),
        assetListResponseJSON: {
            if #available(macOS 13.3, *) {
                return jsonString(fromJSONObject: event.assetListResponse)
            }
            return nil
        }(),
        timelineOccupancyRaw: {
            if #available(macOS 15.0, *) {
                return event.timelineOccupancy.rawValue
            }
            return nil
        }(),
        supplementsPrimaryContent: {
            if #available(macOS 15.0, *) {
                return event.supplementsPrimaryContent
            }
            return nil
        }(),
        contentMayVary: {
            if #available(macOS 15.0, *) {
                return event.contentMayVary
            }
            return nil
        }(),
        hasPrimaryItem: event.primaryItem != nil
    )
}

private func encodeInterstitialMonitorState(_ monitor: AVPlayerInterstitialEventMonitor) -> PlayerInterstitialMonitorStatePayload {
    PlayerInterstitialMonitorStatePayload(
        events: monitor.events.map(encodeInterstitialEventSummary),
        currentEvent: monitor.currentEvent.map(encodeInterstitialEventSummary),
        currentEventSkippableStateRaw: {
            if #available(macOS 26.0, *) {
                return monitor.currentEventSkippableState.rawValue
            }
            return nil
        }(),
        currentEventSkipControlLabel: {
            if #available(macOS 26.0, *) {
                return monitor.currentEventSkipControlLabel
            }
            return nil
        }()
    )
}

private func interstitialCueString(_ cue: AVPlayerInterstitialEvent.Cue) -> String {
    switch cue {
    case .noCue:
        return "no_cue"
    case .joinCue:
        return "join_cue"
    case .leaveCue:
        return "leave_cue"
    default:
        return cue.rawValue
    }
}

private func interstitialCue(from raw: String) -> AVPlayerInterstitialEvent.Cue {
    switch raw {
    case "no_cue":
        return .noCue
    case "join_cue":
        return .joinCue
    case "leave_cue":
        return .leaveCue
    default:
        return AVPlayerInterstitialEvent.Cue(rawValue: raw)
    }
}

private func jsonString(fromJSONObject object: Any?) -> String? {
    guard let object else { return nil }
    guard JSONSerialization.isValidJSONObject(object) else { return nil }
    guard let data = try? JSONSerialization.data(withJSONObject: object) else { return nil }
    return String(data: data, encoding: .utf8)
}
