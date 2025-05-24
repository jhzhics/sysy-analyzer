/**
 * @file A simple parser for sysy language
 * @author jhzhics <zhangjiahao2022@stu.pku.edu.cn>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "sysy_parser",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
