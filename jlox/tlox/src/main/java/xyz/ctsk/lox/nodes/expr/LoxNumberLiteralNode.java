package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.frame.VirtualFrame;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

public class LoxNumberLiteralNode extends LoxExpressionNode {
    private final double value;

    public LoxNumberLiteralNode(double value) {
        this.value = value;
    }

    public double executeDouble(VirtualFrame frame) {
        return this.value;
    }

    @Override
    public Object executeGeneric(VirtualFrame frame) {
        return this.value;
    }
}
