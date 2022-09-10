package xyz.ctsk.lox;

public class Return extends RuntimeException {
    final Object value;

    Return(Object value) {
        super(null, null, false, false);
        this.value = value;
    }

    /* Potentially much faster function calls
    @Override
    public synchronized Throwable fillInStackTrace() {
        return this;
    }
    */
}
