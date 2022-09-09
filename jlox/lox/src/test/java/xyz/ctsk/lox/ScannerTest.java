package xyz.ctsk.lox;

import org.junit.jupiter.api.Test;

import java.util.List;

import static xyz.ctsk.lox.TokenType.*;

import static org.junit.jupiter.api.Assertions.*;

class ScannerTest {
    @Test
    void scanTokens_allTypes() {
        var TEST_STRING = """
              var return print ; nil
              * == or - { 10203
              ( class = false / < if true
              ) "test string" != for
              . and } this <= ident + fun , while
              >= else ! super >
              """;

        var expected = List.of(
                VAR, RETURN, PRINT, SEMICOLON, NIL,
                STAR, EQUAL_EQUAL, OR, MINUS, LEFT_BRACE, NUMBER,
                LEFT_PAREN, CLASS, EQUAL, FALSE, SLASH, LESS, IF, TRUE,
                RIGHT_PAREN, STRING, BANG_EQUAL, FOR,
                DOT, AND, RIGHT_BRACE, THIS, LESS_EQUAL, IDENTIFIER, PLUS, FUN, COMMA, WHILE,
                GREATER_EQUAL, ELSE, BANG, SUPER, GREATER, EOF
        );

        var scanner = new Scanner(TEST_STRING);
        var tokens = scanner.scanTokens();
        var types = tokens.stream().map(Token::type).toList();

        assertEquals(types, expected);
    }

    @Test
    void scanTokens_number() {
        var numberToken = new Scanner("192304").scanTokens().get(0);
        assertEquals(numberToken.type(), NUMBER);
        assertEquals((double) numberToken.literal(), 192304.0);
    }

    @Test
    void scanTokens_string() {
        var testString = """
                "this is a string"
                """;
        var stringToken = new Scanner(testString).scanTokens().get(0);

        assertEquals(stringToken.type(), STRING);
        assertEquals((String) stringToken.literal(), "this is a string");
    }
}