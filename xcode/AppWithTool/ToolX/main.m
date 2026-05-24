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

void ToolXLogStatus(void) {
    NSLog(@"%@", ToolXCopyStatus());
}
