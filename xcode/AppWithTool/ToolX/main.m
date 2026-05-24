//
//  main.m
//  ToolX
//
//  Created by git on 4/25/23.
//

#import "ToolX.h"
#import <TargetConditionals.h>

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
#import <Cocoa/Cocoa.h>
#import <Foundation/Foundation.h>
#include "AppWithTool/execcl_bridge.h"
#include "AppWithTool/execv_bridge.h"
#include "AppWithTool/logargc.h"
#include "AppWithTool/logargv.h"

static void report(void) {
    NSLog(@"ToolX:report()");
}

static void runScriptSharedSupport(NSString *scriptName) {
    NSTask *task = [[NSTask alloc] init];
    [task setLaunchPath:@"/bin/sh"];

    NSString *newpath = [NSString stringWithFormat:@"%@/%@", [[NSBundle mainBundle] sharedSupportPath], scriptName];
    NSLog(@"shell script path: %@", newpath);
    [task setArguments:@[newpath]];

    NSPipe *pipe = [NSPipe pipe];
    [task setStandardOutput:pipe];

    NSFileHandle *file = [pipe fileHandleForReading];

    [task launch];

    NSData *data = [file readDataToEndOfFile];
    NSString *string = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
    NSLog(@"script returned:\n%@", string);
}
#endif

void ToolXRun(int argc, const char *argv[]) {
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    @autoreleasepool {
        report();

        int *count = &argc;
        NSLog(@"logargc(count)");
        int returncount = logargc(count);
        NSLog(@"returncount = logargc(count) = %d", returncount);

        execv_bridge(argv[0], (char **)argv);
        execcl_bridge(argc, (char **)argv);

        runScriptSharedSupport(@"template.sh");
        runScriptSharedSupport(@"Script.sh");
    }
#else
    (void)argc;
    (void)argv;
#endif
}
