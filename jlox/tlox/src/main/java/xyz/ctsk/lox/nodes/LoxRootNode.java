package xyz.ctsk.lox.nodes;

import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.nodes.RootNode;

public class LoxRootNode extends RootNode {
    @SuppressWarnings("FieldMayBeFinal")
    @Child
    private LoxExpressionNode exprNode;

    public LoxRootNode(LoxExpressionNode exprNode) {
        super(null);
        this.exprNode = exprNode;
    }

    @Override
    public Object execute(VirtualFrame frame) {
        return this.exprNode.executeGeneric(frame);
    }
}
