package xyz.ctsk.lox.nodes;

import com.oracle.truffle.api.frame.FrameDescriptor;
import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.nodes.RootNode;

public class LoxRootNode extends RootNode {
    @SuppressWarnings("FieldMayBeFinal")
    @Child
    private LoxExpressionNode exprNode;

    public LoxRootNode(LoxExpressionNode exprNode, FrameDescriptor frameDescriptor) {
        super(null, frameDescriptor);
        this.exprNode = exprNode;
    }

    @Override
    public Object execute(VirtualFrame frame) {
        return this.exprNode.executeGeneric(frame);
    }
}
