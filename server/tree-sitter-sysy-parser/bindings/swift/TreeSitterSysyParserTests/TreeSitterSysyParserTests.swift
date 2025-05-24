import XCTest
import SwiftTreeSitter
import TreeSitterSysyParser

final class TreeSitterSysyParserTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_sysy_parser())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading SysyParser grammar")
    }
}
