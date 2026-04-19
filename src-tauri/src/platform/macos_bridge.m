#import "macos_bridge.h"

#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <dispatch/dispatch.h>
#import <stdlib.h>

void MRMediaRemoteGetNowPlayingInfo(dispatch_queue_t queue, void (^handler)(NSDictionary *info));

char *waken_frontmost_app_name(void) {
    NSRunningApplication *frontmostApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    if (!frontmostApp) return NULL;

    NSString *localizedName = frontmostApp.localizedName;
    if (!localizedName || localizedName.length == 0) return NULL;

    return strdup([localizedName UTF8String]);
}

char *waken_frontmost_app_bundle_identifier(void) {
    NSRunningApplication *frontmostApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    if (!frontmostApp) return NULL;

    NSString *bundleIdentifier = frontmostApp.bundleIdentifier;
    if (!bundleIdentifier || bundleIdentifier.length == 0) return NULL;

    return strdup([bundleIdentifier UTF8String]);
}

char *waken_frontmost_window_title(void) {
    NSRunningApplication *frontmostApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    if (!frontmostApp) return NULL;

    pid_t pid = frontmostApp.processIdentifier;
    if (pid <= 0) return NULL;

    AXUIElementRef appElement = AXUIElementCreateApplication(pid);
    if (!appElement) return NULL;

    CFTypeRef focusedWindow = NULL;
    AXError windowError = AXUIElementCopyAttributeValue(
        appElement,
        kAXFocusedWindowAttribute,
        &focusedWindow
    );

    char *result = NULL;
    if (windowError == kAXErrorSuccess && focusedWindow) {
        CFTypeRef titleValue = NULL;
        AXError titleError = AXUIElementCopyAttributeValue(
            (AXUIElementRef)focusedWindow,
            kAXTitleAttribute,
            &titleValue
        );

        if (titleError == kAXErrorSuccess && titleValue) {
            if (CFGetTypeID(titleValue) == CFStringGetTypeID()) {
                NSString *title = (__bridge NSString *)titleValue;
                NSString *trimmed = [title stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
                if (trimmed.length > 0) {
                    result = strdup([trimmed UTF8String]);
                }
            }
            CFRelease(titleValue);
        }

        CFRelease(focusedWindow);
    }

    CFRelease(appElement);
    return result;
}

bool waken_accessibility_is_trusted(void) {
    return AXIsProcessTrusted();
}

bool waken_request_accessibility_permission(void) {
    NSDictionary *options = @{
        (__bridge NSString *)kAXTrustedCheckOptionPrompt : @YES
    };
    return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}

char *waken_media_now_playing_json(void) {
    dispatch_semaphore_t sem = dispatch_semaphore_create(0);
    __block char *result = NULL;

    MRMediaRemoteGetNowPlayingInfo(
        dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0),
        ^(NSDictionary *info) {
            if (!info) {
                dispatch_semaphore_signal(sem);
                return;
            }

            NSString *title = info[@"kMRMediaRemoteNowPlayingInfoTitle"] ?: @"";
            NSString *artist = info[@"kMRMediaRemoteNowPlayingInfoArtist"] ?: @"";
            NSString *album = info[@"kMRMediaRemoteNowPlayingInfoAlbum"] ?: @"";

            NSDictionary *payload = @{
                @"title": title,
                @"artist": artist,
                @"album": album,
                @"sourceAppId": @"MediaRemote"
            };

            NSError *error = nil;
            NSData *jsonData = [NSJSONSerialization dataWithJSONObject:payload options:0 error:&error];
            if (!error && jsonData) {
                NSString *jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
                result = strdup([jsonString UTF8String]);
            }
            dispatch_semaphore_signal(sem);
        }
    );

    dispatch_semaphore_wait(sem, dispatch_time(DISPATCH_TIME_NOW, (int64_t)(2 * NSEC_PER_SEC)));
    return result;
}

char *waken_bundle_icon_png_base64(const char *bundle_identifier, int target_size) {
    if (bundle_identifier == NULL) return NULL;

    NSString *bundleIdentifier = [NSString stringWithUTF8String:bundle_identifier];
    if (!bundleIdentifier || bundleIdentifier.length == 0) return NULL;

    NSWorkspace *workspace = [NSWorkspace sharedWorkspace];
    NSURL *appURL = [workspace URLForApplicationWithBundleIdentifier:bundleIdentifier];
    NSImage *icon = nil;

    if (appURL.path.length > 0) {
        icon = [workspace iconForFile:appURL.path];
    }

    if (!icon) {
        NSArray<NSRunningApplication *> *runningApps =
            [NSRunningApplication runningApplicationsWithBundleIdentifier:bundleIdentifier];
        if (runningApps.count > 0) {
            icon = runningApps.firstObject.icon;
        }
    }

    if (!icon) return NULL;

    CGFloat clampedTarget = MAX(32, MIN(target_size, 1024));
    NSSize outputSize = NSMakeSize(clampedTarget, clampedTarget);
    NSRect outputRect = NSMakeRect(0, 0, outputSize.width, outputSize.height);

    NSBitmapImageRep *imageRep = [[NSBitmapImageRep alloc]
        initWithBitmapDataPlanes:NULL
                      pixelsWide:(NSInteger)outputSize.width
                      pixelsHigh:(NSInteger)outputSize.height
                   bitsPerSample:8
                 samplesPerPixel:4
                        hasAlpha:YES
                        isPlanar:NO
                  colorSpaceName:NSCalibratedRGBColorSpace
                    bytesPerRow:0
                   bitsPerPixel:0];
    if (!imageRep) return NULL;

    NSGraphicsContext *context = [NSGraphicsContext graphicsContextWithBitmapImageRep:imageRep];
    if (!context) return NULL;

    [NSGraphicsContext saveGraphicsState];
    [NSGraphicsContext setCurrentContext:context];
    [[NSGraphicsContext currentContext] setImageInterpolation:NSImageInterpolationHigh];

    NSImageRep *bestRep = [icon bestRepresentationForRect:outputRect context:nil hints:nil];
    if (bestRep) {
        [bestRep drawInRect:outputRect];
    } else {
        [icon drawInRect:outputRect
                fromRect:NSZeroRect
               operation:NSCompositingOperationSourceOver
                fraction:1.0];
    }

    [NSGraphicsContext restoreGraphicsState];

    NSData *pngData = [imageRep representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
    if (!pngData || pngData.length == 0) return NULL;

    NSString *base64 = [pngData base64EncodedStringWithOptions:0];
    if (!base64 || base64.length == 0) return NULL;

    return strdup([base64 UTF8String]);
}

char *waken_bundle_display_name(const char *bundle_identifier) {
    if (bundle_identifier == NULL) return NULL;

    NSString *bundleIdentifier = [NSString stringWithUTF8String:bundle_identifier];
    if (!bundleIdentifier || bundleIdentifier.length == 0) return NULL;

    NSWorkspace *workspace = [NSWorkspace sharedWorkspace];
    NSURL *appURL = [workspace URLForApplicationWithBundleIdentifier:bundleIdentifier];
    NSString *displayName = nil;

    if (appURL.path.length > 0) {
        displayName = [[NSFileManager defaultManager] displayNameAtPath:appURL.path];
    }

    if (!displayName || displayName.length == 0) {
        NSArray<NSRunningApplication *> *runningApps =
            [NSRunningApplication runningApplicationsWithBundleIdentifier:bundleIdentifier];
        if (runningApps.count > 0) {
            displayName = runningApps.firstObject.localizedName;
        }
    }

    if (!displayName || displayName.length == 0) return NULL;

    return strdup([displayName UTF8String]);
}

void waken_string_free(char *value) {
    if (value != NULL) {
        free(value);
    }
}
