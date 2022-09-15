package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.NodeField;
import com.oracle.truffle.api.dsl.Specialization;
import com.oracle.truffle.api.frame.VirtualFrame;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

@NodeField(name = "slot", type = int.class)
public abstract class LoxReadVariableNode extends LoxExpressionNode {
    protected abstract int getSlot();

    @Specialization(guards = "frame.isDouble(getSlot())")
    protected double readDouble(VirtualFrame frame) {
        return frame.getLong(getSlot());
    }

    @Specialization(guards = "frame.isBoolean(getSlot())")
    protected boolean readBoolean(VirtualFrame frame) {
        return frame.getBoolean(getSlot());
    }

    @Specialization(replaces = { "readDouble", "readBoolean"})
    protected Object readObject(VirtualFrame frame) {
        return frame.getObject(getSlot());
    }
}
