package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.Specialization;
import xyz.ctsk.lox.runtime.LoxException;
import xyz.ctsk.lox.nodes.LoxBinaryNode;

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
