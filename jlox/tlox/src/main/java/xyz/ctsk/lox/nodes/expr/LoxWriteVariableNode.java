package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.NodeChild;
import com.oracle.truffle.api.dsl.NodeField;
import com.oracle.truffle.api.dsl.Specialization;
import com.oracle.truffle.api.frame.FrameSlotKind;
import com.oracle.truffle.api.frame.VirtualFrame;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

@NodeChild("valueNode")
@NodeField(name = "slot", type = int.class)
public abstract class LoxWriteVariableNode extends LoxExpressionNode {
    protected abstract int getSlot();

    @Specialization(guards = "isDoubleOrIllegal(frame)")
    protected double writeDouble(VirtualFrame frame, double value) {
        frame.getFrameDescriptor().setSlotKind(getSlot(), FrameSlotKind.Double);
        frame.setDouble(getSlot(), value);
        return value;
    }

    @Specialization(guards = "isBooleanOrIllegal(frame)")
    protected boolean writeBoolean(VirtualFrame frame, boolean value) {
        frame.getFrameDescriptor().setSlotKind(getSlot(), FrameSlotKind.Boolean);
        frame.setBoolean(getSlot(), value);
        return value;
    }

    @Specialization(replaces = { "writeDouble", "writeBoolean"})
    protected Object write(VirtualFrame frame, Object value) {
        frame.getFrameDescriptor().setSlotKind(getSlot(), FrameSlotKind.Object);
        frame.setObject(getSlot(), value);
        return value;
    }

    protected boolean isDoubleOrIllegal(VirtualFrame frame) {
        final FrameSlotKind kind = frame.getFrameDescriptor().getSlotKind(getSlot());
        return kind == FrameSlotKind.Double || kind == FrameSlotKind.Illegal;
    }

    protected boolean isBooleanOrIllegal(VirtualFrame frame) {
        final FrameSlotKind kind = frame.getFrameDescriptor().getSlotKind(getSlot());
        return kind == FrameSlotKind.Boolean || kind == FrameSlotKind.Illegal;
    }
}
