package xyz.ctsk.lox.parser;

import org.antlr.v4.runtime.Token;
import xyz.ctsk.lox.LoxException;
import xyz.ctsk.lox.nodes.LoxExpressionNode;
import xyz.ctsk.lox.nodes.expr.*;

public class LoxNodeFactory {
    public static LoxNumberLiteralNode createNumberLiteral(Token literalToken) {
        var value = Double.parseDouble(literalToken.getText());
        return new LoxNumberLiteralNode(value);
    }

    public static LoxExpressionNode createUnary(Token op, LoxExpressionNode value) {
        return switch (op.getText()) {
            case "-" -> LoxNegNodeGen.create(value);
            case "!" -> LoxLogicalNotNodeGen.create(value);
            default -> null;
        };
    }

    public static LoxExpressionNode createBinary(Token op, LoxExpressionNode left, LoxExpressionNode right) {
        return switch (op.getText()) {
            case "+" -> LoxAddNodeGen.create(left, right);
            case "-" -> LoxSubNodeGen.create(left, right);
            case "*" -> LoxMulNodeGen.create(left, right);
            case "/" -> LoxDivNodeGen.create(left, right);
            case "<" -> LoxLessNodeGen.create(left, right);
            case "<=" -> LoxLessOrEqualNodeGen.create(left, right);
            case ">" -> LoxGreaterNodeGen.create(left, right);
            case ">=" -> LoxGreaterOrEqualNodeGen.create(left, right);
            case "==" -> LoxEqualNodeGen.create(left, right);
            case "!=" -> LoxLogicalNotNodeGen.create(LoxEqualNodeGen.create(left, right));
            default -> null;
        };
    }
}
