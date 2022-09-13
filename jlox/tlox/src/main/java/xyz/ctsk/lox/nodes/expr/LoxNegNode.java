package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.Specialization;
import xyz.ctsk.lox.LoxException;
import xyz.ctsk.lox.nodes.LoxUnaryNode;

public abstract class LoxNegNode extends LoxUnaryNode {
    @Specialization
    public double negate(double value) {
        return -value;
    }

    @Fallback
    protected Object typeError(Object value) {
        throw LoxException.typeError(this, value);
    }
}
