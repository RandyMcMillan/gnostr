//
//  main.m
//  AppWithTool
//
//  Created by git on 4/25/23.
//

#import "AppDelegate.h"
#import "ToolX/ToolX.h"
#import <TargetConditionals.h>

int main(int argc, char const *argv[]) {
    @autoreleasepool {
        ToolXRun(argc, argv);
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
        return NSApplicationMain(argc, argv);
#else
        return UIApplicationMain(argc, argv, nil, NSStringFromClass([AppDelegate class]));
#endif
    }
}
