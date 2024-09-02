package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.Specialization;
import com.oracle.truffle.api.strings.TruffleString;
import xyz.ctsk.lox.runtime.LoxException;
import xyz.ctsk.lox.nodes.LoxBinaryNode;

public abstract class LoxAddNode extends LoxBinaryNode {
    @Specialization
    public double add(double left, double right) {
        return left + right;
    }

    @Specialization
    public TruffleString add(TruffleString left, TruffleString right) {
        return TruffleString.ConcatNode.create().execute(left, right, TruffleString.Encoding.UTF_16, false);
    }

    @Fallback
    protected Object typeError(Object left, Object right) {
        throw LoxException.typeError(this, left, right);
    }
}
