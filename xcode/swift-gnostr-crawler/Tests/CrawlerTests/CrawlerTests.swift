import Foundation
import Testing
@testable import Crawler
@testable import GnostrTypes

@Test func relayDiscoveryDecodes() throws {
    let json = #"""
    [{
      "url":"wss://relay.example",
      "contact":"ops@example.com",
      "description":"Relay description",
      "name":"Relay name",
      "software":"strfry",
      "version":"1.0.0",
      "supported_nips":[1,11,50],
      "supported_nip_extensions":["nip50-search"],
      "source_nips":[1,11,50]
    }]
    """#

    let entries = try JSONDecoder().decode([RelayDiscoveryEntry].self, from: Data(json.utf8))
    #expect(entries.count == 1)
    #expect(entries[0].supportedNips == [1, 11, 50])
    #expect(entries[0].supportedNipExtensions == ["nip50-search"])
    #expect(entries[0].sourceNips == [1, 11, 50])
}

@Test func relayProcessStateDecodes() throws {
    let json = #"{"running":true,"pid":1234,"message":"ok","disk_usage_bytes":99}"#
    let state = try JSONDecoder().decode(RelayProcessState.self, from: Data(json.utf8))
    #expect(state.running)
    #expect(state.pid == 1234)
    #expect(state.diskUsageBytes == 99)
}

@Test func queryBuilderMatchesCrawlerShape() throws {
    let wire = try CrawlerQueryBuilder.buildGnostrQuery(
        authors: "aa11, bb22",
        ids: "deadbeef",
        limit: 25,
        generic: (tag: "k", value: "1"),
        hashtag: "gnostr",
        mentions: "ee11",
        references: "ff22",
        kinds: "kind:1,nip=1617",
        search: ("search", "ignored")
    )

    let decoded = try JSONDecoder().decode(ClientMessage.self, from: Data(wire.utf8))
    let expectedFilter = Filter(
        ids: [Id(hex: "deadbeef")],
        authors: [PublicKey(hex: "aa11"), PublicKey(hex: "bb22")],
        kinds: [.textNote, .patches],
        tags: ["#k": ["1"], "#t": ["gnostr"], "#p": ["ee11"], "#e": ["ff22"]],
        limit: 25
    )
    #expect(decoded == .req(SubscriptionId("gnostr-query"), [expectedFilter]))
}

@Test func queryURLBuildsExpectedPath() {
    let parameters = CrawlerQueryParameters(
        relay: "wss://relay.example",
        authors: "aa11",
        ids: "deadbeef",
        limit: 5,
        genericTag: "k",
        genericValue: "1",
        hashtag: "gnostr",
        mentions: "ee11",
        references: "ff22",
        kinds: "1,1617",
        search: "hello"
    )

    let url = parameters.queryURL(baseURL: URL(string: "http://127.0.0.1:3030")!, nip: 34)
    #expect(url.path == "/34/query")
    let items = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems ?? []
    #expect(items.contains(URLQueryItem(name: "relay", value: "wss://relay.example")))
    #expect(items.contains(URLQueryItem(name: "search", value: "hello")))
}
