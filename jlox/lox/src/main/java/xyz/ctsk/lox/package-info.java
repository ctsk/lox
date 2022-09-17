@Grammar({
        @Root(name = "Expr",
                rules = {
                        @Rule(head = "Assign", body = {"Token name", "Expr value"}),
                        @Rule(head = "Binary", body = {"Expr left", "Token operator", "Expr right"}),
                        @Rule(head = "Get", body = {"Expr object", "Token name"}),
                        @Rule(head = "Call", body = {"Expr callee", "Token paren", "List<Expr> arguments"}),
                        @Rule(head = "Grouping", body = {"Expr expression"}),
                        @Rule(head = "Literal", body = {"Object value"}),
                        @Rule(head = "Logical", body = {"Expr left", "Token operator", "Expr right"}),
                        @Rule(head = "Set", body = {"Expr object", "Token name", "Expr value"}),
                        @Rule(head = "This", body = {"Token keyword"}),
                        @Rule(head = "Unary", body = {"Token operator", "Expr right"}),
                        @Rule(head = "Variable", body = {"Token name"})
                }),
        @Root(name = "Stmt",
                rules = {
                        @Rule(head = "Block", body = {"List<Stmt> statements"}),
                        @Rule(head = "Class", body = {"Token name", "List<Stmt.Function> methods"}),
                        @Rule(head = "Expression", body = {"Expr expression"}),
                        @Rule(head = "Function", body = {"Token name", "List<Token> params", "List<Stmt> body"}),
                        @Rule(head = "If", body = {"Expr condition", "Stmt thenBranch", "Stmt elseBranch"}),
                        @Rule(head = "Print", body = {"Expr expression"}),
                        @Rule(head = "Return", body = {"Token keyword", "Expr value"}),
                        @Rule(head = "Var", body = {"Token name", "Expr initializer"}),
                        @Rule(head = "While", body = {"Expr condition", "Stmt body"})
                })
})
package xyz.ctsk.lox;

import xyz.ctsk.lox.codegen.Grammar;

import static xyz.ctsk.lox.codegen.Grammar.Rule;
import static xyz.ctsk.lox.codegen.Grammar.Root;