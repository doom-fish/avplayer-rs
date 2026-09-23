#import "AVPlayerObjCBridge.h"

static void AVPStoreReason(NSException *exception, NSString * _Nullable * _Nullable reason) {
    if (reason != NULL) {
        *reason = exception.reason ?: exception.name;
    }
}

CMSampleBufferRef _Nullable AVPTryCopyNextSampleBuffer(
    AVAssetReaderOutput *output,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [output copyNextSampleBuffer];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NULL;
    }
}

BOOL AVPTrySetAlwaysCopiesSampleData(
    AVAssetReaderOutput *output,
    BOOL alwaysCopiesSampleData,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        output.alwaysCopiesSampleData = alwaysCopiesSampleData;
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}

BOOL AVPTrySetSupportsRandomAccess(
    AVAssetReaderOutput *output,
    BOOL supportsRandomAccess,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        output.supportsRandomAccess = supportsRandomAccess;
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}

BOOL AVPTryResetForReadingTimeRanges(
    AVAssetReaderOutput *output,
    NSArray<NSValue *> *timeRanges,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        [output resetForReadingTimeRanges:timeRanges];
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}

AVTimedMetadataGroup * _Nullable AVPTryNextTimedMetadataGroup(
    AVAssetReaderOutputMetadataAdaptor *adaptor,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [adaptor nextTimedMetadataGroup];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

AVCaptionGroup * _Nullable AVPTryNextCaptionGroup(
    AVAssetReaderOutputCaptionAdaptor *adaptor,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [adaptor nextCaptionGroup];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

AVAssetReaderOutputMetadataAdaptor * _Nullable AVPTryCreateMetadataAdaptor(
    AVAssetReaderTrackOutput *trackOutput,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [[AVAssetReaderOutputMetadataAdaptor alloc] initWithAssetReaderTrackOutput:trackOutput];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

AVAssetReaderOutputCaptionAdaptor * _Nullable AVPTryCreateCaptionAdaptor(
    AVAssetReaderTrackOutput *trackOutput,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [[AVAssetReaderOutputCaptionAdaptor alloc] initWithAssetReaderTrackOutput:trackOutput];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

AVAssetReaderTrackOutput * _Nullable AVPTryCreateTrackOutput(
    AVAssetTrack *track,
    NSDictionary<NSString *, id> * _Nullable outputSettings,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [[AVAssetReaderTrackOutput alloc] initWithTrack:track outputSettings:outputSettings];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

AVAssetReaderAudioMixOutput * _Nullable AVPTryCreateAudioMixOutput(
    NSArray<AVAssetTrack *> *audioTracks,
    NSDictionary<NSString *, id> * _Nullable audioSettings,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [[AVAssetReaderAudioMixOutput alloc] initWithAudioTracks:audioTracks audioSettings:audioSettings];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

AVAssetReaderVideoCompositionOutput * _Nullable AVPTryCreateVideoCompositionOutput(
    NSArray<AVAssetTrack *> *videoTracks,
    NSDictionary<NSString *, id> * _Nullable videoSettings,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [[AVAssetReaderVideoCompositionOutput alloc] initWithVideoTracks:videoTracks videoSettings:videoSettings];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

AVPlayer * _Nullable AVPTryCreatePlayerWithItem(
    AVPlayerItem *item,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [[AVPlayer alloc] initWithPlayerItem:item];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

BOOL AVPTryReplaceCurrentItem(
    AVPlayer *player,
    AVPlayerItem * _Nullable item,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        [player replaceCurrentItemWithPlayerItem:item];
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}

AVQueuePlayer * _Nullable AVPTryCreateQueuePlayerWithItems(
    NSArray<AVPlayerItem *> *items,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        return [[AVQueuePlayer alloc] initWithItems:items];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

BOOL AVPTryInsertItem(
    AVQueuePlayer *player,
    AVPlayerItem *item,
    AVPlayerItem * _Nullable afterItem,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        [player insertItem:item afterItem:afterItem];
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}

AVPlayerLooper * _Nullable AVPTryCreatePlayerLooper(
    AVQueuePlayer *player,
    AVPlayerItem *templateItem,
    CMTimeRange timeRange,
    NSInteger itemOrdering,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        if (itemOrdering >= 0) {
            if (@available(macOS 14.0, *)) {
                return [[AVPlayerLooper alloc] initWithPlayer:player
                                                 templateItem:templateItem
                                                    timeRange:timeRange
                                        existingItemsOrdering:(AVPlayerLooperItemOrdering)itemOrdering];
            }
            if (reason != NULL) {
                *reason = @"existingItemsOrdering requires macOS 14.0+";
            }
            return nil;
        }
        return [[AVPlayerLooper alloc] initWithPlayer:player templateItem:templateItem timeRange:timeRange];
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return nil;
    }
}

BOOL AVPTryPreroll(
    AVPlayer *player,
    float rate,
    void (^completion)(BOOL finished),
    NSString * _Nullable * _Nullable reason
) {
    @try {
        [player prerollAtRate:rate completionHandler:completion];
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}

BOOL AVPTrySetRateTimeAtHostTime(
    AVPlayer *player,
    float rate,
    CMTime itemTime,
    CMTime hostClockTime,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        [player setRate:rate time:itemTime atHostTime:hostClockTime];
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}

BOOL AVPTrySetVideoComposition(
    AVPlayerItem *item,
    AVVideoComposition * _Nullable videoComposition,
    NSString * _Nullable * _Nullable reason
) {
    @try {
        item.videoComposition = videoComposition;
        return YES;
    } @catch (NSException *exception) {
        AVPStoreReason(exception, reason);
        return NO;
    }
}
