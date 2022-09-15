package xyz.ctsk.lox.parser;

import com.oracle.truffle.api.frame.FrameDescriptor;
import com.oracle.truffle.api.frame.FrameSlotKind;
import com.oracle.truffle.api.strings.TruffleString;
import org.antlr.v4.runtime.Token;
import xyz.ctsk.lox.nodes.LoxExpressionNode;
import xyz.ctsk.lox.nodes.expr.*;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

public class LoxNodeFactory {
    private static class GlobalScope {
        private final Map<TruffleString, Integer> values = new HashMap<>();
        private final FrameDescriptor.Builder builder = FrameDescriptor.newBuilder();

        Integer find(TruffleString name) {
            return values.get(name);
        }

        int add(TruffleString name) {
            var slot = builder.addSlot(FrameSlotKind.Illegal, name, null);
            values.put(name, slot);
            return slot;
        }

        Integer findOrAdd(TruffleString name) {
            return Optional.ofNullable(find(name)).orElseGet(() -> add(name));
        }

        FrameDescriptor getFrame() {
            return builder.build();
        }
    }

    private final GlobalScope globalScope = new GlobalScope();
    public LoxNumberLiteralNode createNumberLiteral(Token literalToken) {
        var value = Double.parseDouble(literalToken.getText());
        return new LoxNumberLiteralNode(value);
    }

    public LoxStringLiteralNode createStringLiteral(Token literalToken) {
        var value = TruffleString.fromJavaStringUncached(literalToken.getText(), TruffleString.Encoding.UTF_16);
        return new LoxStringLiteralNode(value);
    }

    public LoxBooleanLiteralNode createBooleanLiteral(Token literalToken) {
        var value = Boolean.parseBoolean(literalToken.getText());
        return new LoxBooleanLiteralNode(value);
    }

    public LoxNilLiteralNode createNilLiteral() {
        return new LoxNilLiteralNode();
    }

    public LoxExpressionNode createUnary(Token op, LoxExpressionNode value) {
        return switch (op.getText()) {
            case "-" -> LoxNegNodeGen.create(value);
            case "!" -> LoxLogicalNotNodeGen.create(value);
            default -> null;
        };
    }

    public LoxExpressionNode createBinary(Token op, LoxExpressionNode left, LoxExpressionNode right) {
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

    public LoxWriteVariableNode createAssignment(Token identifier, LoxExpressionNode value) {
        var name = TruffleString.fromJavaStringUncached(identifier.getText(), TruffleString.Encoding.US_ASCII);
        var slot = globalScope.findOrAdd(name);
        return LoxWriteVariableNodeGen.create(value, slot);
    }

    public LoxReadVariableNode createRead(Token identifier) {
        var name = TruffleString.fromJavaStringUncached(identifier.getText(), TruffleString.Encoding.US_ASCII);
        var slot = globalScope.find(name);
        return LoxReadVariableNodeGen.create(slot);
    }

    public FrameDescriptor getFrameDescriptor() {
        return globalScope.getFrame();
    }
}
