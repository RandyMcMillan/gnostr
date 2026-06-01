import Foundation
import SwiftUI
import WebKit

#if canImport(UIKit)
import UIKit

public struct CrawlerServerWebView: UIViewRepresentable {
    public let url: URL

    public init(url: URL) {
        self.url = url
    }

    public func makeUIView(context: Context) -> WKWebView {
        let webView = WKWebView()
        webView.navigationDelegate = context.coordinator
        webView.allowsBackForwardNavigationGestures = true
        return webView
    }

    public func updateUIView(_ webView: WKWebView, context: Context) {
        context.coordinator.load(url: self.url, in: webView)
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    public final class Coordinator: NSObject, WKNavigationDelegate {
        private var currentURL: URL?

        func load(url: URL, in webView: WKWebView) {
            guard self.currentURL != url else { return }
            self.currentURL = url
            webView.load(URLRequest(url: url))
        }
    }
}
#elseif canImport(AppKit)
import AppKit

public struct CrawlerServerWebView: NSViewRepresentable {
    public let url: URL

    public init(url: URL) {
        self.url = url
    }

    public func makeNSView(context: Context) -> WKWebView {
        let webView = WKWebView()
        webView.navigationDelegate = context.coordinator
        return webView
    }

    public func updateNSView(_ webView: WKWebView, context: Context) {
        context.coordinator.load(url: self.url, in: webView)
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    public final class Coordinator: NSObject, WKNavigationDelegate {
        private var currentURL: URL?

        func load(url: URL, in webView: WKWebView) {
            guard self.currentURL != url else { return }
            self.currentURL = url
            webView.load(URLRequest(url: url))
        }
    }
}
#endif
