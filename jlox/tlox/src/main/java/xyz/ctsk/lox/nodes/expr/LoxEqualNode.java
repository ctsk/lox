package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.Specialization;
import xyz.ctsk.lox.nodes.LoxBinaryNode;

import java.util.Objects;

public abstract class LoxEqualNode extends LoxBinaryNode {
    @Specialization
    public boolean equal(double left, double right) {
        return left == right;
    }

    @Specialization
    public boolean equal(boolean left, boolean right) {
        return left == right;
    }

    @Fallback
    protected Object typeError(Object left, Object right) {
        return Objects.equals(left, right);
    }

}
