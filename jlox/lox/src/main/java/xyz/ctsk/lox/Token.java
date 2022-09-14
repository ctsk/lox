package xyz.ctsk.lox;

public record Token(TokenType type, String lexeme, Object literal, int line, int position) {
    public Token(TokenType type, String lexeme, int line, int position) {
        this(type, lexeme, null, line, position);
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
