@Grammar({
        @Root(name = "Expr",
                rules = {
                        @Rule(head = "Binary", body = {"Expr left", "Token operator", "Expr right"}),
                        @Rule(head = "Grouping", body = {"Expr expression"}),
                        @Rule(head = "Literal", body = {"Object value"}),
                        @Rule(head = "Unary", body = {"Token operator", "Expr right"})
                }),
        @Root(name = "Stmt",
                rules = {
                        @Rule(head = "Expression", body = "Expr expression"),
                        @Rule(head = "Print", body = "Expr expression")
                })
})
package xyz.ctsk.lox;

import xyz.ctsk.lox.codegen.Grammar;

import static xyz.ctsk.lox.codegen.Grammar.Rule;
import static xyz.ctsk.lox.codegen.Grammar.Root;