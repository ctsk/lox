package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.nodes.Node;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

public class LoxAddNode extends LoxExpressionNode {
    @SuppressWarnings("FieldMayBeFinal")
    @Node.Child
    private LoxExpressionNode leftNode, rightNode;

    public LoxAddNode(LoxExpressionNode leftNode, LoxExpressionNode rightNode) {
        this.leftNode = leftNode;
        this.rightNode = rightNode;
    }


    @Override
    public double executeDouble(VirtualFrame frame) {
        var leftValue = leftNode.executeDouble(frame);
        var rightValue = rightNode.executeDouble(frame);
        return leftValue + rightValue;
    }
}
