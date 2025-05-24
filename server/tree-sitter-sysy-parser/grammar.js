/**
 * @file A simple parser for sysy language
 * @author jhzhics <zhangjiahao2022@stu.pku.edu.cn>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "sysy_parser",

  extras: $ => [
    $.comment,
    /\s/
  ],

  conflicts: $ => [
    // Add any conflicts here if needed
  ],

  precedences: $ => [
    // Precedence from lowest to highest
    [
      'logical_or',
      'logical_and',
      'equality',
      'relational',
      'additive',
      'multiplicative',
      'unary',
    ],
  ],

  rules: {
    source_file: $ => optional($.CompUnit),

    comment: $ => token(choice(
      /\/\/[^\n]*/,
      /\/\*[^*]*\*+([^/*][^*]*\*+)*\//
    )),

    CompUnit: $ => repeat1(choice($.Decl, $.FuncDef)),
    
    Type: $ => choice(
      "int",
      "void"),

    FuncDef: $ => seq(
      $.Type,
      $.Ident,
      "(",
      optional($.FuncFParams),
      ")",
      $.Block
    ),

    FuncFParams: $ => seq(
      $.FuncFParam,
      repeat(seq(",", $.FuncFParam))
    ),

    FuncFParam: $ => seq(
      $.Type,
      $.Ident,
      optional(seq(
        "[",
        "]",
        repeat(seq("[", $.ConstExp, "]"))
      ))
    ),

    Decl: $ => choice(
      $.VarDecl,
      $.ConstDecl
    ),

    VarDecl: $ => seq(
      $.Type,
      $.VarDef,
      repeat(seq(",", $.VarDef)),
      ";"
    ),

    ConstDecl: $ => seq(
      "const",
      $.Type,
      $.ConstDef,
      repeat(seq(",", $.ConstDef)),
      ";"
    ),

    ConstDef: $ => seq(
      $.Ident,
      repeat(seq("[", $.ConstExp, "]")),
      "=",
      $.ConstInitVal
    ),

    ConstInitVal: $ => choice(
      $.ConstExp,
      seq(
        "{",
        optional(seq(
          $.ConstInitVal,
          repeat(seq(",", $.ConstInitVal))
        )),
        "}"
      )
    ),

    ConstExp: $ => $.Exp,

    VarDef: $ => choice(
      seq(
        $.Ident,
        repeat(seq("[", $.Exp, "]"))
      ),
      seq(
        $.Ident,
        repeat(seq("[", $.Exp, "]")),
        "=",
        $.InitVal
      )
    ),

    InitVal: $ => choice(
      $.Exp,
      seq(
        "{",
        optional(seq(
          $.InitVal,
          repeat(seq(",", $.InitVal))
        )),
        "}"
      )
    ),

    Lval: $ => seq($.Ident, repeat(seq("[", $.Exp, "]"))),

    PrimaryExp: $ => choice(
      $.Number,
      $.Lval,
      seq("(", $.Exp, ")")
    ),

    Stmt: $ => choice(
      seq($.Lval, "=", $.Exp, ";"),
      seq(optional($.Exp), ";"),
      $.Block,
      prec.right(seq("if", "(", $.Exp, ")", $.Stmt, optional(seq("else", $.Stmt)))),
      seq("while", "(", $.Exp, ")", $.Stmt),
      seq("break", ";"),
      seq("continue", ";"),
      seq("return", optional($.Exp), ";")
    ),

    BlockItem: $ => choice(
      $.Decl,
      $.Stmt
    ),

    Block: $ => seq(
      "{",
      repeat($.BlockItem),
      "}"
    ),

    FuncRParams: $ => seq(
      $.Exp,
      repeat(seq(",", $.Exp))
    ),
    
    Exp: $ => choice(
      $.PrimaryExp,
      seq(
        $.Ident,
        "(",
        optional($.FuncRParams),
        ")"
      ),
      prec.right('unary', seq("+", $.Exp)),
      prec.right('unary', seq("-", $.Exp)),
      prec.right('unary', seq("!", $.Exp)),
      prec.left('additive', seq($.Exp, "+", $.Exp)),
      prec.left('additive', seq($.Exp, "-", $.Exp)),
      prec.left('multiplicative', seq($.Exp, "*", $.Exp)),
      prec.left('multiplicative', seq($.Exp, "/", $.Exp)),
      prec.left('multiplicative', seq($.Exp, "%", $.Exp)),
      prec.left('relational', seq($.Exp, "<", $.Exp)),
      prec.left('relational', seq($.Exp, ">", $.Exp)),
      prec.left('relational', seq($.Exp, "<=", $.Exp)),
      prec.left('relational', seq($.Exp, ">=", $.Exp)),
      prec.left('equality', seq($.Exp, "==", $.Exp)),
      prec.left('equality', seq($.Exp, "!=", $.Exp)),
      prec.left('logical_and', seq($.Exp, "&&", $.Exp)),
      prec.left('logical_or', seq($.Exp, "||", $.Exp)),
    ),

    Number: $ => choice(
      $.Decimal,
      $.Octal,
      $.Hexadecimal
    ),
    Ident: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    Decimal: $ => /[1-9][0-9]*/,
    Octal: $ => /0[0-7]*/,
    Hexadecimal: $ => /0[xX][0-9a-fA-F]+/,
  }
});
