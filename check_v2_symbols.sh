#!/bin/bash
SDK=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk

# Sample of key symbols from v1 audit
symbols_to_check=(
    "AVPlayer"
    "AVPlayerItem"
    "AVPlayerStatus"
    "AVQueuePlayer"
    "AVPlayerLooper"
    "AVPlayerInterstitialEventController"
    "AVPlayerLayer"
    "AVAsset"
    "AVPlayerLooperStatus"
    "AVPlayerVideoOutput"
    "AVPlayerWaitingReason"
    "AVPlayerRateDidChangeNotification"
)

framework_path="$SDK/System/Library/Frameworks/AVFoundation.framework/Headers"

found=0
missing=0

for symbol in "${symbols_to_check[@]}"; do
    # Search in all headers
    if grep -r "$symbol" "$framework_path" > /dev/null 2>&1; then
        echo "✓ $symbol found"
        ((found++))
    else
        echo "✗ $symbol NOT FOUND"
        ((missing++))
    fi
done

echo ""
echo "Summary: $found found, $missing missing"
