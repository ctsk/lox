package xyz.ctsk.lox.parser;

import com.oracle.truffle.api.strings.TruffleString;
import org.antlr.v4.runtime.Token;
import xyz.ctsk.lox.nodes.LoxExpressionNode;
import xyz.ctsk.lox.nodes.expr.*;

public class LoxNodeFactory {
    public static LoxNumberLiteralNode createNumberLiteral(Token literalToken) {
        var value = Double.parseDouble(literalToken.getText());
        return new LoxNumberLiteralNode(value);
    }

    public static LoxStringLiteralNode createStringLiteral(Token literalToken) {
        var text = literalToken.getText();
        var value = TruffleString.fromJavaStringUncached(text.substring(1, text.length() - 1), TruffleString.Encoding.UTF_16);
        return new LoxStringLiteralNode(value);
    }

    public static LoxBooleanLiteralNode createBooleanLiteral(Token literalToken) {
        var value = Boolean.parseBoolean(literalToken.getText());
        return new LoxBooleanLiteralNode(value);
    }

    public static LoxNilLiteralNode createNilLiteral() {
        return new LoxNilLiteralNode();
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
