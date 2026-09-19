import XCTest
@testable import SkillsCore

final class TokenCounterTests: XCTestCase {

    let counter = TokenCounter()

    func testEmptyStringIsZero() {
        XCTAssertEqual(counter.estimate(text: ""), 0)
    }

    func testSingleWord() {
        XCTAssertEqual(counter.estimate(text: "hello"), 1)
    }

    func testTwoWords() {
        // words=2, spaces=1 → 3
        XCTAssertEqual(counter.estimate(text: "hello world"), 3)
    }

    func testMultilineContent() {
        let text = "hello world\nfoo bar baz"
        // words: hello, world, foo, bar, baz = 5; spaces = 4 → 9
        XCTAssertEqual(counter.estimate(text: text), 9)
    }

    func testWhitespaceOnlyIsZero() {
        XCTAssertEqual(counter.estimate(text: "   \n\t  "), 0)
    }
}
