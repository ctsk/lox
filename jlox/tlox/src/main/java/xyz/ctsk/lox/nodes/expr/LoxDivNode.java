package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.Specialization;
import xyz.ctsk.lox.LoxException;

public abstract class LoxDivNode extends LoxBinaryNode {
    @Specialization
    public double div(double left, double right) {
        return left / right;
    }

    @Fallback
    protected Object typeError(Object left, Object right) {
        throw LoxException.typeError(this, left, right);
    }

}
