//
//  main.m
//  AppWithTool
//
//  Created by git on 4/25/23.
//

//REF: https://developer.apple.com/forums/thread/106590

//main.h
#import <Foundation/Foundation.h>
#import <Cocoa/Cocoa.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include "execv_bridge.h"
#include "execcl_bridge.h"
#include "async.h/async/async.h"

//int execv(const char *path, char *const argv[]);
//int execve(const char *path, char *const argv[], char *const envp[]);
//int execvp(const char *file, char *const argv[]);
//int execvpe(const char *file, char *const argv[], char *const envp[]);

void runScript(NSString* scriptName);
//main.h end
//#import main.h

int execve(const char *path, char *const argv[], char *const envp[]){ return 0; }
int execvp(const char *file, char *const argv[]){ return 0; }
int execvpe(const char *file, char *const argv[], char *const envp[]){ return 0; }

void privateFrameworksPath(NSString* scriptName)
{
    NSTask *task;
    task = [[NSTask alloc] init];
    [task setLaunchPath: @"/bin/sh"];

    NSArray *arguments;
    NSString* newpath = [NSString stringWithFormat:@"%@/%@",[[NSBundle mainBundle] privateFrameworksPath], scriptName];
    //NSString* newpath = [NSString stringWithFormat:@"%@/%@",[[NSBundle mainBundle] sharedSupportPath], scriptName];
    NSLog(@"shell script path: %@", newpath);
    arguments = [NSArray arrayWithObjects:newpath, nil];
    [task setArguments: arguments];

    NSPipe *pipe;
    pipe = [NSPipe pipe];
    [task setStandardOutput: pipe];

    NSFileHandle *file;
    file = [pipe fileHandleForReading];

    [task launch];

    NSData *data;
    data = [file readDataToEndOfFile];

    NSString *string;
    string = [[NSString alloc] initWithData: data encoding: NSUTF8StringEncoding];
    NSLog (@"script returned:\n%@", string);
}



void runScriptSharedSupport(NSString* scriptName)
{
    NSTask *task;
    task = [[NSTask alloc] init];
    [task setLaunchPath: @"/bin/sh"];

    NSArray *arguments;
    //NSString* newpath = [NSString stringWithFormat:@"%@/%@",[[NSBundle mainBundle] privateFrameworksPath], scriptName];
    NSString* newpath = [NSString stringWithFormat:@"%@/%@",[[NSBundle mainBundle] sharedSupportPath], scriptName];
    NSLog(@"shell script path: %@", newpath);
    arguments = [NSArray arrayWithObjects:newpath, nil];
    [task setArguments: arguments];

    NSPipe *pipe;
    pipe = [NSPipe pipe];
    [task setStandardOutput: pipe];

    NSFileHandle *file;
    file = [pipe fileHandleForReading];

    [task launch];

    NSData *data;
    data = [file readDataToEndOfFile];

    NSString *string;
    string = [[NSString alloc] initWithData: data encoding: NSUTF8StringEncoding];
    NSLog (@"script returned:\n%@", string);
}

int main(int argc, char const *argv[]) {
    @autoreleasepool {
        NSLog (@"main");
        int *count=&argc;
        NSLog (@"logargc(count)");
        int returncount = logargc(count);
        NSLog (@"returncount = logargc(count) = %d", returncount);

        argv[argc] = "ToolX";//(int *)argc;
        //logargc(count);
        //logargc(&argc);
        
        //int success = logargv(&argc, (char **)argv);//blocking
        //if(success){printf("success=%d",success);};
        //if(!success){printf("!success=%d",success);};
        
        execv_bridge((const char *)argv, (char **)argv);
        //int execcl_bridge(int argc, char* argv[]);
        execcl_bridge(argc,(char **)argv);
        
        runScriptSharedSupport(@"template.sh");
        runScriptSharedSupport(@"Script.sh");

    }
    return NSApplicationMain(argc, argv);
}
