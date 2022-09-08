package xyz.ctsk.lox;

public record Token(TokenType type, String lexeme, Object literal, int line) {
    public Token(TokenType type, String lexeme, int line) {
        this(type, lexeme, null, line);
    }

    @Override
    public String toString() {
        if (literal == null) {
            return "%s %s".formatted(type, lexeme);
        } else {
            return "%s %s %s".formatted(type, lexeme, literal);
        }
    }
}
