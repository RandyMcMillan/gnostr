//
//  ViewController.m
//  AppWithTool
//
//  Created by git on 4/25/23.
//

#import "ViewController.h"

@implementation ViewController

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
- (void)viewDidLoad {
    [super viewDidLoad];

    // Do any additional setup after loading the view.
}


- (void)setRepresentedObject:(id)representedObject {
    [super setRepresentedObject:representedObject];

    // Update the view, if already loaded.
}
#else
- (void)viewDidLoad {
    [super viewDidLoad];
}
#endif


@end
