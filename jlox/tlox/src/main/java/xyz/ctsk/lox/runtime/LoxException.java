package xyz.ctsk.lox.runtime;

import com.oracle.truffle.api.exception.AbstractTruffleException;
import com.oracle.truffle.api.nodes.Node;

public class LoxException extends AbstractTruffleException {

    public LoxException(String message) {
        super(message);
    }
    public static LoxException typeError(Node operation, Object... values) {
        return new LoxException(operation.toString());
    }
}
