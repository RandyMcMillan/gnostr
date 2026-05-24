//
//  ViewController.m
//  AppWithTool
//
//  Created by git on 4/25/23.
//

#import "ViewController.h"
#import "ToolX/ToolX.h"
#import <TargetConditionals.h>

@interface ViewController ()

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
@property (strong, nonatomic) NSTextField *statusLabel;
@property (strong, nonatomic) NSButton *refreshButton;
@property (strong, nonatomic) NSTextView *logView;
#else
@property (strong, nonatomic) UILabel *statusLabel;
@property (strong, nonatomic) UIButton *refreshButton;
@property (strong, nonatomic) UITextView *logView;
#endif

@end

@implementation ViewController

- (void)loadView {
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.view = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 900, 640)];
#else
    self.view = [[UIView alloc] initWithFrame:[UIScreen mainScreen].bounds];
#endif
}

- (void)viewDidLoad {
    [super viewDidLoad];
    self.view.translatesAutoresizingMaskIntoConstraints = NO;

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.statusLabel = [NSTextField labelWithString:@""];
    self.statusLabel.alignment = NSTextAlignmentCenter;
    self.statusLabel.font = [NSFont systemFontOfSize:18 weight:NSFontWeightSemibold];
    self.statusLabel.lineBreakMode = NSLineBreakByWordWrapping;
    self.statusLabel.maximumNumberOfLines = 0;

    self.refreshButton = [NSButton buttonWithTitle:@"Refresh ToolX" target:self action:@selector(refreshToolX:)];
    self.refreshButton.bezelStyle = NSBezelStyleRounded;
    self.logView = [[NSTextView alloc] initWithFrame:NSZeroRect];
    self.logView.editable = NO;
    self.logView.selectable = YES;
    self.logView.richText = NO;
    self.logView.font = [NSFont monospacedSystemFontOfSize:13 weight:NSFontWeightRegular];
#else
    self.statusLabel = [[UILabel alloc] initWithFrame:CGRectZero];
    self.statusLabel.numberOfLines = 0;
    self.statusLabel.textAlignment = NSTextAlignmentCenter;
    self.statusLabel.font = [UIFont systemFontOfSize:20 weight:UIFontWeightSemibold];
    self.statusLabel.textColor = [UIColor labelColor];

    self.refreshButton = [UIButton buttonWithType:UIButtonTypeSystem];
    [self.refreshButton setTitle:@"Refresh ToolX" forState:UIControlStateNormal];
    self.refreshButton.titleLabel.font = [UIFont systemFontOfSize:17 weight:UIFontWeightSemibold];
    [self.refreshButton addTarget:self action:@selector(refreshToolX:) forControlEvents:UIControlEventTouchUpInside];
    self.logView = [[UITextView alloc] initWithFrame:CGRectZero];
    self.logView.editable = NO;
    self.logView.selectable = YES;
    self.logView.font = [UIFont monospacedSystemFontOfSize:13 weight:UIFontWeightRegular];
    self.logView.backgroundColor = [UIColor secondarySystemBackgroundColor];
#endif

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.view.wantsLayer = YES;
    self.view.layer.backgroundColor = NSColor.windowBackgroundColor.CGColor;
#else
    self.view.backgroundColor = [self backgroundColor];
#endif
    [self.view addSubview:self.statusLabel];
    [self.view addSubview:self.refreshButton];
    [self.view addSubview:self.logView];

    [self installConstraints];
    [self refreshToolX:nil];
}

- (void)refreshToolX:(id)sender {
    NSString *status = ToolXCopyStatus();
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.statusLabel.stringValue = status;
#else
    self.statusLabel.text = status;
#endif
    ToolXLogStatus();
    [self appendLogLine:[NSString stringWithFormat:@"[%@] %@", [self currentTimestamp], status]];
}

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
- (NSColor *)backgroundColor {
    return NSColor.windowBackgroundColor;
}
#else
- (UIColor *)backgroundColor {
    return UIColor.systemBackgroundColor;
}
#endif

- (void)installConstraints {
    self.statusLabel.translatesAutoresizingMaskIntoConstraints = NO;
    self.refreshButton.translatesAutoresizingMaskIntoConstraints = NO;
    self.logView.translatesAutoresizingMaskIntoConstraints = NO;

    [NSLayoutConstraint activateConstraints:@[
        [self.statusLabel.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor constant:24],
        [self.statusLabel.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor constant:-24],
        [self.statusLabel.topAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.topAnchor constant:24],
        [self.refreshButton.topAnchor constraintEqualToAnchor:self.statusLabel.bottomAnchor constant:24],
        [self.refreshButton.centerXAnchor constraintEqualToAnchor:self.view.centerXAnchor],
        [self.logView.topAnchor constraintEqualToAnchor:self.refreshButton.bottomAnchor constant:24],
        [self.logView.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor constant:24],
        [self.logView.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor constant:-24],
        [self.logView.bottomAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.bottomAnchor constant:-24],
    ]];
}

- (void)appendLogLine:(NSString *)line {
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    NSTextStorage *storage = self.logView.textStorage;
    if (storage.length > 0) {
        [storage appendAttributedString:[[NSAttributedString alloc] initWithString:@"\n"]];
    }
    [storage appendAttributedString:[[NSAttributedString alloc] initWithString:line]];
    [self.logView scrollRangeToVisible:NSMakeRange(storage.length, 0)];
#else
    NSString *existing = self.logView.text ?: @"";
    if (existing.length > 0) {
        existing = [existing stringByAppendingString:@"\n"];
    }
    self.logView.text = [existing stringByAppendingString:line];
    NSRange range = NSMakeRange(self.logView.text.length, 0);
    [self.logView scrollRangeToVisible:range];
#endif
}

- (NSString *)currentTimestamp {
    NSDateFormatter *formatter = [[NSDateFormatter alloc] init];
    formatter.locale = [NSLocale localeWithLocaleIdentifier:@"en_US_POSIX"];
    formatter.dateFormat = @"yyyy-MM-dd HH:mm:ss";
    return [formatter stringFromDate:[NSDate date]];
}

@end
