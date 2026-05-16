import AVFoundation
import Foundation

@_cdecl("av_player_item_access_log_release")
public func av_player_item_access_log_release(_ logPtr: UnsafeMutableRawPointer?) {
    guard let logPtr else { return }
    Unmanaged<AVPlayerItemAccessLog>.fromOpaque(logPtr).release()
}

@_cdecl("av_player_item_access_log_info_json")
public func av_player_item_access_log_info_json(
    _ logPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let log = Unmanaged<AVPlayerItemAccessLog>.fromOpaque(logPtr).takeUnretainedValue()
    let payload = AccessLogPayload(
        extendedLog: avpString(from: log.extendedLogData(), encoding: log.extendedLogDataStringEncoding),
        extendedLogDataStringEncoding: UInt(log.extendedLogDataStringEncoding),
        events: log.events.map(encodeAccessLogEvent)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_player_item_error_log_release")
public func av_player_item_error_log_release(_ logPtr: UnsafeMutableRawPointer?) {
    guard let logPtr else { return }
    Unmanaged<AVPlayerItemErrorLog>.fromOpaque(logPtr).release()
}

@_cdecl("av_player_item_error_log_info_json")
public func av_player_item_error_log_info_json(
    _ logPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let log = Unmanaged<AVPlayerItemErrorLog>.fromOpaque(logPtr).takeUnretainedValue()
    let payload = ErrorLogPayload(
        extendedLog: avpString(from: log.extendedLogData(), encoding: log.extendedLogDataStringEncoding),
        extendedLogDataStringEncoding: UInt(log.extendedLogDataStringEncoding),
        events: log.events.map(encodeErrorLogEvent)
    )
    do {
        return ffiString(try avpEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

private func encodeAccessLogEvent(_ event: AVPlayerItemAccessLogEvent) -> AccessLogEventPayload {
    AccessLogEventPayload(
        numberOfMediaRequests: event.numberOfMediaRequests,
        playbackStartDate: avpISO8601String(event.playbackStartDate),
        uri: event.uri,
        serverAddress: event.serverAddress,
        numberOfServerAddressChanges: event.numberOfServerAddressChanges,
        playbackSessionId: event.playbackSessionID,
        playbackStartOffset: event.playbackStartOffset,
        segmentsDownloadedDuration: event.segmentsDownloadedDuration,
        durationWatched: event.durationWatched,
        numberOfStalls: event.numberOfStalls,
        numberOfBytesTransferred: Int64(event.numberOfBytesTransferred),
        transferDuration: event.transferDuration,
        observedBitrate: event.observedBitrate,
        indicatedBitrate: event.indicatedBitrate,
        indicatedAverageBitrate: event.indicatedAverageBitrate,
        averageVideoBitrate: event.averageVideoBitrate,
        averageAudioBitrate: event.averageAudioBitrate,
        numberOfDroppedVideoFrames: event.numberOfDroppedVideoFrames,
        startupTime: event.startupTime,
        downloadOverdue: event.downloadOverdue,
        observedBitrateStandardDeviation: event.observedBitrateStandardDeviation,
        playbackType: event.playbackType,
        mediaRequestsWwan: event.mediaRequestsWWAN,
        switchBitrate: event.switchBitrate
    )
}

private func encodeErrorLogEvent(_ event: AVPlayerItemErrorLogEvent) -> ErrorLogEventPayload {
    let allHttpResponseHeaderFields: [String: String]?
    if #available(macOS 14.5, *) {
        allHttpResponseHeaderFields = event.allHTTPResponseHeaderFields
    } else {
        allHttpResponseHeaderFields = nil
    }

    return ErrorLogEventPayload(
        date: avpISO8601String(event.date),
        uri: event.uri,
        serverAddress: event.serverAddress,
        playbackSessionId: event.playbackSessionID,
        errorStatusCode: event.errorStatusCode,
        errorDomain: event.errorDomain,
        errorComment: event.errorComment,
        allHttpResponseHeaderFields: allHttpResponseHeaderFields
    )
}
