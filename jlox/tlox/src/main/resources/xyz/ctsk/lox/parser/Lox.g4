grammar Lox;


@parser::header {

import com.oracle.truffle.api.source.Source;

import xyz.ctsk.lox.nodes.LoxExpressionNode;
import xyz.ctsk.lox.parser.*;

}

@parser::members {
    private LoxNodeFactory factory;

    public static Object parseLox(String source) {
        LoxLexer lexer = new LoxLexer(CharStreams.fromString(source));
        LoxParser parser = new LoxParser(new CommonTokenStream(lexer));

        parser.factory = new LoxNodeFactory();

        return parser.expression();
    }
}

file returns [LoxExpressionNode result]
    : expression EOF { $result = $expression.result; }
    ;

expression returns [LoxExpressionNode result]
    : literal                                 { $result = $literal.result; }
    | left=expression op=( '*' | '/' ) right=expression { $result = factory.createBinaryNode($op, $left.result, $right.result); }
    | left=expression op=( '+' | '-' ) right=expression { $result = factory.createBinaryNode($op, $left.result, $right.result); }
    ;

literal returns [LoxExpressionNode result]
    : NUMBER { $result = factory.createNumberLiteral($NUMBER); }
    ;

AND: 'and' ;
CLASS: 'class' ;
ELSE: 'else' ;
FALSE: 'false' ;
FOR: 'for' ;
FUN: 'fun' ;
IF: 'if' ;
NIL: 'nil' ;
OR: 'or' ;
PRINT: 'print' ;
RETURN: 'return' ;
SUPER: 'super' ;
THIS: 'this' ;
TRUE: 'true' ;
VAR: 'var' ;
WHILE: 'while';

LESS: '<' ;
LESS_EQUAL: '<=';
GREATER: '>' ;
GREATER_EQUAL: '>=' ;
BANG: '!' ;
BANG_EQUAL: '!=' ;
EQUAL: '=' ;
EQUAL_EQUAL: '==' ;

NUMBER
    : DIGIT+ ('.' DIGIT+)?
    ;

STRING
    : '"' ~["]* '"'
    ;

IDENTIFIER
    : LETTER ( LETTER | DIGIT | UNDERSCORE )*
    ;

fragment LETTER
    : [a-zA-Z]
    ;

fragment DIGIT
    : [0-9]
    ;

fragment UNDERSCORE
    : '_'
    ;

WS
    : [ \t\r\n]+ -> skip
    ;