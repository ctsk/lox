package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.frame.VirtualFrame;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

public class LoxNilLiteralNode extends LoxExpressionNode {
    @Override
    public Object executeGeneric(VirtualFrame frame) {
        return null;
    }
}
