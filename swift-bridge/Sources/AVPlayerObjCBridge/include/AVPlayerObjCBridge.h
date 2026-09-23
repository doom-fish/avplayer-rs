#import <AVFoundation/AVFoundation.h>
#import <CoreMedia/CoreMedia.h>

NS_ASSUME_NONNULL_BEGIN

CMSampleBufferRef _Nullable AVPTryCopyNextSampleBuffer(
    AVAssetReaderOutput *output,
    NSString * _Nullable * _Nullable reason
) CF_RETURNS_RETAINED;

BOOL AVPTrySetAlwaysCopiesSampleData(
    AVAssetReaderOutput *output,
    BOOL alwaysCopiesSampleData,
    NSString * _Nullable * _Nullable reason
);

BOOL AVPTrySetSupportsRandomAccess(
    AVAssetReaderOutput *output,
    BOOL supportsRandomAccess,
    NSString * _Nullable * _Nullable reason
);

BOOL AVPTryResetForReadingTimeRanges(
    AVAssetReaderOutput *output,
    NSArray<NSValue *> *timeRanges,
    NSString * _Nullable * _Nullable reason
);

AVTimedMetadataGroup * _Nullable AVPTryNextTimedMetadataGroup(
    AVAssetReaderOutputMetadataAdaptor *adaptor,
    NSString * _Nullable * _Nullable reason
);

AVCaptionGroup * _Nullable AVPTryNextCaptionGroup(
    AVAssetReaderOutputCaptionAdaptor *adaptor,
    NSString * _Nullable * _Nullable reason
) API_AVAILABLE(macos(12.0));

AVAssetReaderOutputMetadataAdaptor * _Nullable AVPTryCreateMetadataAdaptor(
    AVAssetReaderTrackOutput *trackOutput,
    NSString * _Nullable * _Nullable reason
);

AVAssetReaderOutputCaptionAdaptor * _Nullable AVPTryCreateCaptionAdaptor(
    AVAssetReaderTrackOutput *trackOutput,
    NSString * _Nullable * _Nullable reason
) API_AVAILABLE(macos(12.0));

AVAssetReaderTrackOutput * _Nullable AVPTryCreateTrackOutput(
    AVAssetTrack *track,
    NSDictionary<NSString *, id> * _Nullable outputSettings,
    NSString * _Nullable * _Nullable reason
);

AVAssetReaderAudioMixOutput * _Nullable AVPTryCreateAudioMixOutput(
    NSArray<AVAssetTrack *> *audioTracks,
    NSDictionary<NSString *, id> * _Nullable audioSettings,
    NSString * _Nullable * _Nullable reason
);

AVAssetReaderVideoCompositionOutput * _Nullable AVPTryCreateVideoCompositionOutput(
    NSArray<AVAssetTrack *> *videoTracks,
    NSDictionary<NSString *, id> * _Nullable videoSettings,
    NSString * _Nullable * _Nullable reason
);

AVPlayer * _Nullable AVPTryCreatePlayerWithItem(
    AVPlayerItem *item,
    NSString * _Nullable * _Nullable reason
);

BOOL AVPTryReplaceCurrentItem(
    AVPlayer *player,
    AVPlayerItem * _Nullable item,
    NSString * _Nullable * _Nullable reason
);

AVQueuePlayer * _Nullable AVPTryCreateQueuePlayerWithItems(
    NSArray<AVPlayerItem *> *items,
    NSString * _Nullable * _Nullable reason
);

BOOL AVPTryInsertItem(
    AVQueuePlayer *player,
    AVPlayerItem *item,
    AVPlayerItem * _Nullable afterItem,
    NSString * _Nullable * _Nullable reason
);

AVPlayerLooper * _Nullable AVPTryCreatePlayerLooper(
    AVQueuePlayer *player,
    AVPlayerItem *templateItem,
    CMTimeRange timeRange,
    NSInteger itemOrdering,
    NSString * _Nullable * _Nullable reason
);

BOOL AVPTryPreroll(
    AVPlayer *player,
    float rate,
    void (^completion)(BOOL finished),
    NSString * _Nullable * _Nullable reason
);

BOOL AVPTrySetRateTimeAtHostTime(
    AVPlayer *player,
    float rate,
    CMTime itemTime,
    CMTime hostClockTime,
    NSString * _Nullable * _Nullable reason
);

BOOL AVPTrySetVideoComposition(
    AVPlayerItem *item,
    AVVideoComposition * _Nullable videoComposition,
    NSString * _Nullable * _Nullable reason
);

NS_ASSUME_NONNULL_END
