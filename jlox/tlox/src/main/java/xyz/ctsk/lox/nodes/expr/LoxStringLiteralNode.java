package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.strings.TruffleString;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

public class LoxStringLiteralNode extends LoxExpressionNode {
    private final TruffleString value;

    public LoxStringLiteralNode(TruffleString value) {
        this.value = value;
    }

    @Override
    public Object executeGeneric(VirtualFrame frame) {
        return value;
    }
}
