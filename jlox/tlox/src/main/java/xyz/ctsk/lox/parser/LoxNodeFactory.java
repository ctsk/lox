package xyz.ctsk.lox.parser;

import org.antlr.v4.runtime.Token;
import xyz.ctsk.lox.nodes.LoxExpressionNode;
import xyz.ctsk.lox.nodes.expr.*;

public class LoxNodeFactory {
    public static LoxNumberLiteralNode createNumberLiteral(Token literalToken) {
        var value = Double.parseDouble(literalToken.getText());
        return new LoxNumberLiteralNode(value);
    }

    public static LoxExpressionNode createBinaryNode(Token op, LoxExpressionNode left, LoxExpressionNode right) {
        return switch (op.getText()) {
            case "+" -> new LoxAddNode(left, right);
            case "-" -> new LoxSubNode(left, right);
            case "*" -> new LoxMulNode(left, right);
            case "/" -> new LoxDivNode(left, right);
            default -> null;
        };
    }
}
