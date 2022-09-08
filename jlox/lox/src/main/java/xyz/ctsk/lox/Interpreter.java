package xyz.ctsk.lox;

public class Interpreter implements Expr.Visitor<Object> {

    void interpret(Expr expression) {
        try {
            Object value = evaluate(expression);
            System.out.println(stringify(value));
        } catch (RuntimeError error) {
            Lox.runtimeError(error);
        }
    }

    private Object evaluate(Expr expr) {
        return expr.accept(this);
    }

    private String stringify(Object object) {
        if (object == null) return "nil";

        var text = object.toString();

        if (object instanceof Double && text.endsWith(".0")) {
            text = text.substring(0, text.length() - 2);
        }
        return text;
    }


    @Override
    public Object visitBinaryExpr(Expr.Binary binary) {
        var left = evaluate(binary.left());
        var right = evaluate(binary.right());

        switch (binary.operator().type()) {
            case MINUS, SLASH, STAR, GREATER, GREATER_EQUAL, LESS, LESS_EQUAL ->
                    checkNumberOperands(binary.operator(), left, right);
        }

        return switch (binary.operator().type()) {
            case MINUS -> (double) left - (double) right;
            case PLUS  -> {
                if (left instanceof Double leftD && right instanceof Double rightD) {
                    yield leftD + rightD;
                }

                if (left instanceof String leftStr && right instanceof String rightStr) {
                    yield leftStr + rightStr;
                }

                throw new RuntimeError(binary.operator(), "Operands must be two numbers or two strings.");
            }
            case SLASH -> (double) left / (double) right;
            case STAR  -> (double) left * (double) right;
            case GREATER       -> (double)left > (double)right;
            case GREATER_EQUAL -> (double)left >= (double)right;
            case LESS          -> (double)left < (double)right;
            case LESS_EQUAL    -> (double)left <= (double)right;
            case BANG_EQUAL  -> !isEqual(left, right);
            case EQUAL_EQUAL -> isEqual(left, right);
            default -> null;
        };
    }

    @Override
    public Object visitGroupingExpr(Expr.Grouping grouping) {
        return evaluate(grouping.expression());
    }

    @Override
    public Object visitLiteralExpr(Expr.Literal literal) {
        return literal.value();
    }

    @Override
    public Object visitUnaryExpr(Expr.Unary unary) {
        var right = evaluate(unary.right());

        return switch(unary.operator().type()) {
            case MINUS -> -asNumber(unary.operator(), right);
            case BANG -> !isTruthy(right);
            default -> null;
        };
    }


    private boolean isTruthy(Object object) {
        if (object == null) return false;
        if (object instanceof Boolean bool) return bool;
        return true;
    }

    private boolean isEqual(Object a, Object b) {
        if (a == null && b == null) return true;
        if (a == null) return false;
        return a.equals(b);
    }

    private double asNumber(Token operator, Object operand) {
        if (operand instanceof Double d) return d;
        throw new RuntimeError(operator, "Operand must be a number.");
    }

    private void checkNumberOperands(Token operator, Object left, Object right) {
        if (left instanceof Double && right instanceof Double) return;
        throw new RuntimeError(operator, "Operands must be numbers");
    }
}
