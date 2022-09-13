package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.frame.VirtualFrame;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

public class LoxDivNode extends LoxExpressionNode {
    @SuppressWarnings("FieldMayBeFinal")
    @Child
    private LoxExpressionNode leftNode, rightNode;

    public LoxDivNode(LoxExpressionNode leftNode, LoxExpressionNode rightNode) {
        this.leftNode = leftNode;
        this.rightNode = rightNode;
    }

    @Override
    public double executeDouble(VirtualFrame frame) {
        var leftValue = leftNode.executeDouble(frame);
        var rightValue = rightNode.executeDouble(frame);
        return leftValue / rightValue;
    }
}
