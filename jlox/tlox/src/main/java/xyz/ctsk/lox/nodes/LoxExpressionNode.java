package xyz.ctsk.lox.nodes;

import com.oracle.truffle.api.dsl.TypeSystem;
import com.oracle.truffle.api.dsl.TypeSystemReference;
import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.nodes.UnexpectedResultException;

@TypeSystemReference(LoxTypes.class)
public abstract class LoxExpressionNode extends LoxNode {
    public double executeDouble(VirtualFrame frame) throws UnexpectedResultException {
        return LoxTypesGen.expectDouble(executeGeneric(frame));
    }

    public boolean executeBoolean(VirtualFrame frame) throws UnexpectedResultException {
        return LoxTypesGen.expectBoolean(executeGeneric(frame));
    }

    public abstract Object executeGeneric(VirtualFrame frame);
}
