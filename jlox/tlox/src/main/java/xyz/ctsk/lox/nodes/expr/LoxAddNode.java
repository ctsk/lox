package xyz.ctsk.lox.nodes.expr;

import com.oracle.truffle.api.dsl.Cached;
import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.Specialization;
import com.oracle.truffle.api.strings.TruffleString;
import xyz.ctsk.lox.LoxLanguage;
import xyz.ctsk.lox.runtime.LoxException;
import xyz.ctsk.lox.nodes.LoxBinaryNode;

public abstract class LoxAddNode extends LoxBinaryNode {
    @Specialization
    public double add(double left, double right) {
        return left + right;
    }

    @Specialization
    public TruffleString add(TruffleString left, TruffleString right, @Cached TruffleString.ConcatNode concatNode) {
        return concatNode.execute(left, right, LoxLanguage.STRING_ENCODING, true);
    }

    @Fallback
    protected Object typeError(Object left, Object right) {
        throw LoxException.typeError(this, left, right);
    }
}
