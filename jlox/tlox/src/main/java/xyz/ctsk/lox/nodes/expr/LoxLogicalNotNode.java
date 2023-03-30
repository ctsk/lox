package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.Specialization;
import xyz.ctsk.lox.runtime.LoxException;
import xyz.ctsk.lox.nodes.LoxUnaryNode;

public abstract class LoxLogicalNotNode extends LoxUnaryNode {
    @Specialization
    public boolean not(boolean value) {
        return !value;
    }

    @Fallback
    protected Object typeError(Object value) {
        throw LoxException.typeError(this, value);
    }
}
