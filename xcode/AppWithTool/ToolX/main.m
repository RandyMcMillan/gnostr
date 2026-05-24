//
//  main.m
//  ToolX
//
//  Created by git on 4/25/23.
//

#import "ToolX.h"
#import <TargetConditionals.h>

NSString *ToolXCopyStatus(void) {
    NSString *platform = @"iOS";

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    platform = @"macOS";
#endif

    NSString *bundleIdentifier = [[NSBundle mainBundle] bundleIdentifier] ?: @"unknown";
    return [NSString stringWithFormat:@"ToolX ready on %@ (%@)", platform, bundleIdentifier];
}

NSArray<NSString *> *ToolXJobNames(void) {
    return @[
        @"status",
        @"bundle-path",
        @"bundle-resources",
        @"environment",
    ];
}

static NSString *ToolXBundlePathJob(void) {
    NSBundle *bundle = [NSBundle mainBundle];
    return [NSString stringWithFormat:@"bundlePath=%@\nresourcePath=%@", bundle.bundlePath, bundle.resourcePath ?: @"(nil)"];
}

static NSString *ToolXBundleResourcesJob(void) {
    NSURL *resourceURL = [[NSBundle mainBundle] resourceURL];
    NSArray<NSURL *> *items = [[NSFileManager defaultManager] contentsOfDirectoryAtURL:resourceURL
                                                           includingPropertiesForKeys:nil
                                                                              options:0
                                                                                error:nil];
    NSMutableArray<NSString *> *names = [NSMutableArray array];
    for (NSURL *item in items) {
        [names addObject:item.lastPathComponent];
    }
    if (names.count == 0) {
        return @"No bundled resources found.";
    }
    return [NSString stringWithFormat:@"Resources:\n%@", [names componentsJoinedByString:@"\n"]];
}

static NSString *ToolXEnvironmentJob(void) {
    NSProcessInfo *processInfo = [NSProcessInfo processInfo];
    return [NSString stringWithFormat:@"process=%@\nhome=%@\nos=%@",
            processInfo.processName,
            NSHomeDirectory(),
            processInfo.operatingSystemVersionString];
}

NSString *ToolXRunJob(NSString *jobName) {
    NSString *normalized = [jobName stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
    if (normalized.length == 0) {
        normalized = @"status";
    }

    NSString *result = nil;
    if ([normalized isEqualToString:@"status"]) {
        result = ToolXCopyStatus();
    } else if ([normalized isEqualToString:@"bundle-path"]) {
        result = ToolXBundlePathJob();
    } else if ([normalized isEqualToString:@"bundle-resources"]) {
        result = ToolXBundleResourcesJob();
    } else if ([normalized isEqualToString:@"environment"]) {
        result = ToolXEnvironmentJob();
    } else {
        result = [NSString stringWithFormat:@"Unknown ToolX job: %@\nAvailable jobs: %@", normalized, [ToolXJobNames() componentsJoinedByString:@", "]];
    }

    NSLog(@"ToolX job %@:\n%@", normalized, result);
    return result;
}

void ToolXLogStatus(void) {
    NSLog(@"%@", ToolXCopyStatus());
}
