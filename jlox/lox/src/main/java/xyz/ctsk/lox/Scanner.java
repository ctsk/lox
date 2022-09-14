package xyz.ctsk.lox;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import static java.util.Map.entry;
import static xyz.ctsk.lox.TokenType.*;

public class Scanner {
    private static final Map<String, TokenType> keywords =
            Map.ofEntries(
                    entry("and", AND),
                    entry("class", CLASS),
                    entry("else", ELSE),
                    entry("false", FALSE),
                    entry("for", FOR),
                    entry("fun", FUN),
                    entry("if", IF),
                    entry("nil", NIL),
                    entry("or", OR),
                    entry("print", PRINT),
                    entry("return", RETURN),
                    entry("super", SUPER),
                    entry("this", THIS),
                    entry("true", TRUE),
                    entry("var", VAR),
                    entry("while", WHILE)
            );

    private final String source;
    private final List<Token> tokens = new ArrayList<>();

    private int start = 0;
    private int current = 0;
    private int line = 1;

    public Scanner(String source) {
        this.source = source;
    }

    private String currentMatch() {
        return source.substring(start, current);
    }

    private void addToken(TokenType type, Object literal) {
        tokens.add(new Token(type, currentMatch(), literal, line, current));
    }

    private void addToken(TokenType type) {
        addToken(type, null);
    }


    private boolean isAtEnd() {
        return current >= source.length();
    }
    private char advance() {
        return source.charAt(current++);
    }
    private char peek() {
        return isAtEnd() ? '\0' : source.charAt(current);
    }
    private boolean match(char expected) {
        if (isAtEnd()) {
            return false;
        }

        if (source.charAt(current) != expected) {
            return false;
        }

        current++;
        return true;
    }

    private void string() {
        while (peek() != '"' && !isAtEnd()) {
            if (peek() == '\n') line++;
            advance();
        }

        if (isAtEnd()) {
            Lox.error(line, "Unterminated string.");
            return;
        }

        advance();

        String value = source.substring(start + 1, current - 1);
        addToken(STRING, value);
    }

    private char peekNext() {
        if (current + 1 >= source.length()) return '\0';
        return source.charAt(current + 1);
    }

    private static boolean isDigit(char c) {
        return '0' <= c && c <= '9';
    }

    private static boolean isAlpha(char c) {
        return ('a' <= c && c <= 'z') ||
                ('A' <= c && c <= 'Z') ||
                c == '_';
    }

    private static boolean isAlphaNumeric(char c) {
        return isDigit(c) || isAlpha(c);
    }

    private void number() {
        while (isDigit(peek())) advance();

        // Look for a fractional part.
        if (peek() == '.' && isDigit(peekNext())) {
            // Consume the "."
            advance();

            while (isDigit(peek())) advance();
        }

        // Enhancement: Do not allow Letters in numbers
        if (isAlpha(peek())) {
            Lox.error(line, "Unexpected character in number.");
            return;
        }

        addToken(NUMBER,
                Double.parseDouble(currentMatch()));
    }


    private void identifier() {
        while (isAlphaNumeric(peek())) advance();

        var text = currentMatch();
        addToken(
                keywords.getOrDefault(text, IDENTIFIER)
        );
    }

    private void scanToken() {
        var c = advance();
        switch (c) {
            case '(' -> addToken(LEFT_PAREN);
            case ')' -> addToken(RIGHT_PAREN);
            case '{' -> addToken(LEFT_BRACE);
            case '}' -> addToken(RIGHT_BRACE);
            case ',' -> addToken(COMMA);
            case '.' -> addToken(DOT);
            case '-' -> addToken(MINUS);
            case '+' -> addToken(PLUS);
            case ';' -> addToken(SEMICOLON);
            case '*' -> addToken(STAR);

            case '!' -> addToken(match('=') ? BANG_EQUAL : BANG);
            case '=' -> addToken(match('=') ? EQUAL_EQUAL : EQUAL);
            case '<' -> addToken(match('=') ? LESS_EQUAL : LESS);
            case '>' -> addToken(match('=') ? GREATER_EQUAL : GREATER);

            case '/' -> {
                if (match('/')) {
                    while (peek() != '\n' && !isAtEnd()) advance();
                } else {
                    addToken(SLASH);
                }
            }

            case ' ', '\r', '\t' -> {}
            case '\n' -> line++;

            case '"' -> string();

            default -> {
                if (isDigit(c)) {
                    number();
                } else if (isAlpha(c)) {
                    identifier();
                } else {
                    Lox.error(line, "Unexpected character.");
                }
            }
        }
    }

    List<Token> scanTokens() {
        while (!isAtEnd()) {
            start = current;
            scanToken();
        }

        tokens.add(new Token(EOF, "", line, current));
        return tokens;
    }
}
