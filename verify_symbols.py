#!/usr/bin/env python3
import re
import subprocess

SDK="/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk"

# Headers in the AVPlayer subsystem
headers = [
    "AVPlayer.h",
    "AVPlayerItem.h",
    "AVPlayerItemOutput.h",
    "AVPlayerLayer.h",
    "AVPlayerLooper.h",
    "AVPlayerInterstitialEventController.h",
    "AVPlayerItemIntegratedTimeline.h",
    "AVPlayerOutput.h",
    "AVPlayerMediaSelectionCriteria.h",
    "AVPlayerItemTrack.h",
    "AVPlayerItemProtectedContentAdditions.h",
    "AVPlayerItemMediaDataCollector.h",
    "AVAsset.h"
]

framework_path = f"{SDK}/System/Library/Frameworks/AVFoundation.framework/Headers"

symbols_by_header = {}

for header in headers:
    filepath = f"{framework_path}/{header}"
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        # Find top-level declarations (non-nested in categories)
        symbols = []
        
        # Typedefs
        typedefs = re.findall(r'^typedef\s+(?:NS_ENUM|NS_OPTIONS|NS_STRING_ENUM)?\(?.*?(?=\);?$)', content, re.MULTILINE)
        symbols.extend(typedefs[:5])  # sample
        
        # Interfaces/Protocols
        interfaces = re.findall(r'^@interface\s+(\w+)', content, re.MULTILINE)
        protocols = re.findall(r'^@protocol\s+(\w+)', content, re.MULTILINE)
        symbols.extend(interfaces)
        symbols.extend(protocols)
        
        # Constants
        constants = re.findall(r'^AVF_EXPORT\s+.*?\s+(AV\w+)\s+(?:const)?', content, re.MULTILINE)
        symbols.extend(constants)
        
        symbols_by_header[header] = len(set(symbols))
        print(f"{header}: {len(set(symbols))} unique symbols")
    except FileNotFoundError:
        print(f"{header}: NOT FOUND")

print(f"\nTotal unique headers checked: {len(symbols_by_header)}")
