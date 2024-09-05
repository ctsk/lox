package xyz.ctsk.lox;

import java.util.ArrayList;
import java.util.List;

import static xyz.ctsk.lox.TokenType.*;

/**
 *
 * A recursive descent parser for the following grammar:
 * <p>
 * program        → declaration* EOF ;
 * declaration    → classDecl
 *                | funDecl
 *                | varDecl
 *                | statement ;
 * classDecl      → "class" IDENTIFIER ( "<" IDENTIFIER )?
 *                  "{" function* "}" ;
 * funDecl        → "fun" function ;
 * function       → IDENTIFIER "(" parameters? ")" block ;
 * parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
 * varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
 * statement      → exprStmt
 *                | forStmt
 *                | ifStmt
 *                | printStmt
 *                | returnStmt
 *                | whileStmt
 *                | block ;
 * block          → "{" declaration* "}" ;
 * forStmt        → "for" "(" (varDecl | exprStmt | ";")
 *                            expression? ";"
 *                            expression? ")" statement ;
 * ifStmt         → "if" "(" expression ")" statement ( "else" statement )? ;
 * whileStmt      → "while" "(" expression ")" statement ;
 * printStmt      → "print" expression ";" ;
 * returnStmt     → "return" expression? ";" ;
 * exprStmt       → expression ";" ;
 * expression     → equality ;
 * expression     → assignment ;
 * assignment     → ( call "." )? IDENTIFIER "=" assignment
 *                | logic_or ;
 * logic_or       → logic_and ( "or" logic_and )* ;
 * logic_and      → equality ( "and" equality )* ;
 * equality       → comparison ( ( "!=" | "==" ) comparison )* ;
 * comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
 * term           → factor ( ( "-" | "+" ) factor )* ;
 * factor         → unary ( ( "/" | "*" ) unary )* ;
 * unary          → ( "!" | "-" ) unary
 *                | call ;
 * call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
 * arguments      → expression ( "," expression )* ;
 * primary        → "true" | "false" | "nil" | "this"
 *                | NUMBER | STRING | IDENTIFIER | "(" expression ")"
 *                | "super" "." IDENTIFIER ;
 */
public class Parser {
    private final List<Token> tokens;

    private int current = 0;

    Parser(List<Token> tokens) {
        this.tokens = tokens;
    }

    List<Stmt> parse() {
        List<Stmt> statements = new ArrayList<>();

        while (!isAtEnd()) {
            statements.add(declaration());
        }

        return statements;
    }

    private Stmt declaration() {
        try {
            if (match(CLASS)) return classDeclaration();
            if (match(FUN)) return function(FunctionType.FUNCTION);
            if (match(VAR)) return varDeclaration();
            return match(VAR) ? varDeclaration() : statement();
        } catch (ParseError error) {
            synchronize();
            return null;
        }
    }

    private Stmt classDeclaration() {
        var name = consume(IDENTIFIER, "Expect class name.");

        Expr.Variable superclass = null;
        if (match(LESS)) {
            consume(IDENTIFIER, "Expect superclass name.");
            superclass = new Expr.Variable(previous());
        }

        consume(LEFT_BRACE, "Expect '{' before class body.");

        List<Stmt.Function> methods = new ArrayList<>();

        while (!check(RIGHT_BRACE) && !isAtEnd()) {
            methods.add(function(FunctionType.METHOD));
        }

        consume(LEFT_BRACE, "Expect '}' after class body.");
        return new Stmt.Class(name, superclass, methods);
    }

    private Stmt.Function function(FunctionType kind) {
        Token name = consume(IDENTIFIER, "Expect %s name.".formatted(kind.toString().toLowerCase()));

        consume(LEFT_PAREN, "Expect '(' after %s name.".formatted(kind.toString().toLowerCase()));
        List<Token> params = new ArrayList<>();
        if (!check(RIGHT_PAREN)) {
            do {
                if (params.size() >= 255) {
                    // DO NOT PANIC
                    //noinspection ThrowableNotThrown
                    error(peek(), "Can't have more than 255 parameters.");
                }
                params.add(consume(IDENTIFIER, "Expect parameter name."));
            } while (match(COMMA));
        }
        consume(RIGHT_PAREN, "Expect ')' after parameters.");

        consume(LEFT_BRACE, "Expect '{' before %s body.".formatted(kind.toString().toLowerCase()));
        List<Stmt> body = blockStatement();
        return new Stmt.Function(name, params, body);
    }

    private Stmt varDeclaration() {
        Token name = consume(IDENTIFIER, "Expect variable name.");

        Expr initializer = null;
        if (match(EQUAL)) {
            initializer = expression();
        }

        consume(SEMICOLON, "Expect ';' after variable declaration.");
        return new Stmt.Var(name, initializer);
    }


    private Stmt statement() {
        if (match(FOR)) return forStatement();
        if (match(IF)) return ifStatement();
        if (match(PRINT)) return printStatement();
        if (match(RETURN)) return returnStatement();
        if (match(WHILE)) return whileStatement();
        if (match(LEFT_BRACE)) return new Stmt.Block(blockStatement());
        return expressionStatement();
    }


    private Stmt forStatement() {
        consume(LEFT_PAREN, "Expect '(' after 'for'.");

        Stmt initializer;
        if (match(SEMICOLON)) {
            initializer = null;
        } else if (match(VAR)) {
            initializer = varDeclaration();
        } else {
            initializer = expressionStatement();
        }

        Expr condition = check(SEMICOLON) ? null : expression();
        consume(SEMICOLON, "Expect ';' after loop condition.");

        Expr increment = check(RIGHT_PAREN) ? null : expression();
        consume(RIGHT_PAREN, "Expect ')' after for clauses.");

        Stmt body = statement();

        if (increment != null) {
            body = new Stmt.Block(List.of(body, new Stmt.Expression(increment)));
        }

        if (condition == null) {
            condition = new Expr.Literal(true);
        }
        body = new Stmt.While(condition, body);

        if (initializer != null) {
            body = new Stmt.Block(List.of(initializer, body));
        }

        return body;
    }


    private Stmt ifStatement() {
        consume(LEFT_PAREN, "Expect '(' after 'if'.");
        Expr condition = expression();
        consume(RIGHT_PAREN, "Expect ')' after condition.");

        Stmt thenBranch = statement();
        Stmt elseBranch = match(ELSE) ? statement() : null;

        return new Stmt.If(condition, thenBranch, elseBranch);
    }

    private Stmt printStatement() {
        Expr value = expression();
        consume(SEMICOLON, "Expect ';' after value.");
        return new Stmt.Print(value);
    }


    private Stmt returnStatement() {
        Token keyword = previous();
        Expr value = null;
        if (!check(SEMICOLON)) {
            value = expression();
        }

        consume(SEMICOLON, "Expect ';' after return value.");
        return new Stmt.Return(keyword, value);
    }

    private Stmt whileStatement() {
        consume(LEFT_PAREN, "Expect '(' after 'while'.");
        Expr condition = expression();
        consume(RIGHT_PAREN, "Expect ')' after condition.");

        Stmt body = statement();

        return new Stmt.While(condition, body);
    }
    private List<Stmt> blockStatement() {
        var statements = new ArrayList<Stmt>();

        while (!check(RIGHT_BRACE) && !isAtEnd()) {
            statements.add(declaration());
        }

        consume(RIGHT_BRACE, "Expect '}' after block.");
        return statements;
    }

    private Stmt expressionStatement() {
        Expr expr = expression();
        consume(SEMICOLON, "Expect ';' after expression.");
        return new Stmt.Expression(expr);
    }

    private Expr expression() {
        return assignment();
    }

    private Expr assignment() {
        Expr expr = or();

        if (match(EQUAL)) {
            Token equals = previous();
            Expr value = assignment();

            if (expr instanceof Expr.Variable varExpr) {
                return new Expr.Assign(varExpr.name(), value);
            } else if (expr instanceof Expr.Get getExpr) {
                return new Expr.Set(getExpr.object(), getExpr.name(), value);
            } else {
                //We report an error but don't throw it because we do not need our parser to panic and sync.
                //noinspection ThrowableNotThrown
                error(equals, "Invalid assignment target.");
            }
        }

        return expr;
    }

    private Expr or() {
        Expr expr = and();

        while (match(OR)) {
            Token operator = previous();
            Expr right = and();
            expr = new Expr.Logical(expr, operator, right);
        }

        return expr;
    }

    private Expr and() {
        Expr expr = equality();

        while (match(AND)) {
            Token operator = previous();
            Expr right = equality();
            expr  = new Expr.Logical(expr, operator, right);
        }

        return expr;
    }

    private Expr equality() {
        Expr expr = comparison();

        while (match(BANG_EQUAL, EQUAL_EQUAL)) {
            Token operator = previous();
            Expr right = comparison();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr comparison() {
        Expr expr = term();

        while (match(GREATER, GREATER_EQUAL, LESS, LESS_EQUAL)) {
            Token operator = previous();
            Expr right = term();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr term() {
        Expr expr = factor();

        while (match(MINUS, PLUS)) {
            Token operator = previous();
            Expr right = factor();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr factor() {
        Expr expr = unary();

        while (match(SLASH, STAR)) {
            Token operator = previous();
            Expr right = unary();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr unary() {
        if (match(BANG, MINUS)) {
            Token operator = previous();
            Expr right = unary();
            return new Expr.Unary(operator, right);
        }

        return call();
    }

    private Expr call() {
        var expr = primary();

        while(true) {
            if (match(LEFT_PAREN)) {
                expr = finishCall(expr);
            } else if (match(DOT)) {
                Token name = consume(IDENTIFIER, "Expect property name after '.'.");
                expr = new Expr.Get(expr, name);
            } else {
                break;
            }
        }

        return expr;
    }

    private Expr finishCall(Expr callee) {
        List<Expr> arguments = new ArrayList<>();

        if (!check(RIGHT_PAREN)) {
            do {
                if (arguments.size() >= 255) {
                    // DO NOT PANIC
                    //noinspection ThrowableNotThrown
                    error(peek(), "Can't have more than 255 arguments.");
                }
                arguments.add(expression());
            } while (match(COMMA));
        }

        Token paren = consume(RIGHT_PAREN, "Expect ')' after arguments.");
        return new Expr.Call(callee, paren, arguments);
    }

    private Expr primary() {
        if (match(FALSE)) return new Expr.Literal(false);
        if (match(TRUE)) return new Expr.Literal(true);
        if (match(NIL)) return new Expr.Literal(null);

        if (match(NUMBER, STRING)) {
            return new Expr.Literal(previous().literal());
        }

        if (match(SUPER)) {
            Token keyword = previous();
            consume(DOT, "Expect '.' after 'super'.");
            Token method = consume(IDENTIFIER, "Expect superclass method name.");
            return new Expr.Super(keyword, method);
        }

        if (match(THIS)) return new Expr.This(previous());

        if (match(IDENTIFIER)) {
            return new Expr.Variable(previous());
        }

        if (match(LEFT_PAREN)) {
            Expr expr = expression();
            consume(RIGHT_PAREN, "Expect ')' after expression.");
            return new Expr.Grouping(expr);
        }

        throw error(peek(), "Expect expression.");
    }

    private boolean isAtEnd() {
        return peek().type() == EOF;
    }

    private Token previous() {
        return tokens.get(current - 1);
    }

    private Token advance() {
        if (!isAtEnd()) current++;
        return previous();
    }

    private Token peek() {
        return tokens.get(current);
    }
    private boolean check(TokenType type) {
        if (isAtEnd()) return false;
        return peek().type() == type;
    }

    private boolean match(TokenType... types) {
        for (var type : types) {
            if (check(type)) {
                advance();
                return true;
            }
        }

        return false;
    }

    private Token consume(TokenType type, String message) {
        if (check(type)) return advance();
        throw error(peek(), message);
    }

    private ParseError error(Token token, String message) {
        Lox.error(token, message);
        return new ParseError();
    }

    private void synchronize() {
        advance();

        while (!isAtEnd()) {
            if (previous().type() == SEMICOLON) return;
            advance();
        }
    }

    private static class ParseError extends RuntimeException {}
}
