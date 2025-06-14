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

  conflicts: _ => [
    // Add any conflicts here if needed
  ],

  precedences: _ => [
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

    CompUnit: $ => repeat1(choice($._Decl, $.FuncDef)),
    
    Type: $ => choice(
      "int",
      "void"),

    FuncDef: $ => seq(
      field("type", $.Type),
      field("ident", $.Ident),
      "(",
      optional(seq(
      field("params", $.FuncFParam),
      repeat(seq(",", field("params", $.FuncFParam)))
      )),
      ")",
      $.Block
    ),

    FuncArraryQualifier: $ => seq(
      "[",
      "]",
      repeat(seq("[", $.ConstExp, "]"))
    ),

    FuncFParam: $ => seq(
      field("type", $.Type),
      field("ident", $.Ident),
      optional(field("array_qualifier", $.FuncArraryQualifier
      ))
    ),

    _Decl: $ => choice(
      $.VarDecl,
      $.ConstDecl
    ),

    VarDecl: $ => seq(
      field("type", $.Type),
      field("defs", $.VarDef),
      repeat(seq(",", field("defs", $.VarDef))),
      ";"
    ),

    ConstDecl: $ => seq(
      "const",
      field("type", $.Type),
      field("defs", $.ConstDef,
      repeat(seq(",", field("defs", $.ConstDef)))),
      ";"
    ),

    ConstArrayQualifier: $ => repeat1(seq("[", $.ConstExp, "]")),

    ConstDef: $ => seq(
      field("ident", $.Ident),
      optional(field("array_qualifier", $.ConstArrayQualifier)),
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

    VarArrayQualifier: $ => repeat1(seq("[", $.Exp, "]")),

    VarDef: $ => choice(
      seq(
        field("ident", $.Ident),
        optional(field("array_qualifier", $.VarArrayQualifier))
      ),
      seq(
        field("ident", $.Ident),
        optional(field("array_qualifier", $.VarArrayQualifier)),
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

    _Stmt: $ => choice(
      seq($.Lval, "=", $.Exp, ";"),
      seq(optional($.Exp), ";"),
      $.Block,
      prec.right(seq("if", "(", $.Exp, ")", $._Stmt, optional(seq("else", $._Stmt)))),
      seq("while", "(", $.Exp, ")", $._Stmt),
      seq("break", ";"),
      seq("continue", ";"),
      seq("return", optional($.Exp), ";")
    ),

    _BlockItem: $ => choice(
      $._Decl,
      $._Stmt
    ),

    Block: $ => seq(
      "{",
      repeat($._BlockItem),
      "}"
    ),

    FuncRParams: $ => seq(
      $.Exp,
      repeat(seq(",", $.Exp))
    ),

    FuncCall: $ => seq(
      $.Ident,
      "(",
      optional($.FuncRParams),
      ")"
    ),
    
    Exp: $ => choice(
      $.PrimaryExp,
      $.FuncCall,
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

    comment: _ => token(choice(
          /\/\*[^*]*\*+([^/*][^*]*\*+)*\//, 
          /\/\/[^\n]*/))
  }
});
