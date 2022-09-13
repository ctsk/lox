package xyz.ctsk.lox.nodes;

import com.oracle.truffle.api.frame.VirtualFrame;

public abstract class LoxExpressionNode extends LoxNode {
    public abstract double executeDouble(VirtualFrame frame);
}
