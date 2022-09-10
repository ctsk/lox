package xyz.ctsk.lox;

import java.util.List;

public class Interpreter implements Expr.Visitor<Object>, Stmt.Visitor<Void> {
    private Environment environment = new Environment();

    void interpret(List<Stmt> statements) {
        try {
            statements.forEach(this::execute);
        } catch (RuntimeError error) {
            Lox.runtimeError(error);
        }
    }

    private void execute(Stmt stmt) {
        stmt.accept(this);
    }

    private void executeBlock(List<Stmt> statements, Environment environment) {
        Environment previous = this.environment;
        try {
            this.environment = environment;

            statements.forEach(this::execute);
        } finally {
            this.environment = previous;
        }
    }

    private Object evaluate(Expr expr) {
        return expr.accept(this);
    }

    static String stringify(Object object) {
        if (object == null) return "nil";

        var text = object.toString();

        if (object instanceof Double && text.endsWith(".0")) {
            text = text.substring(0, text.length() - 2);
        }
        return text;
    }

    @Override
    public Void visitBlockStmt(Stmt.Block stmt) {
        executeBlock(stmt.statements(), new Environment(environment));
        return null;
    }

    @Override
    public Void visitExpressionStmt(Stmt.Expression stmt) {
        evaluate(stmt.expression());
        return null;
    }

    @Override
    public Void visitIfStmt(Stmt.If stmt) {
        if (isTruthy(stmt.condition())) {
            execute(stmt.thenBranch());
        } else if (stmt.elseBranch() != null){
            execute(stmt.elseBranch());
        }
        return null;
    }

    @Override
    public Void visitPrintStmt(Stmt.Print stmt) {
        Object value = evaluate(stmt.expression());
        System.out.println(stringify(value));
        return null;
    }

    @Override
    public Void visitVarStmt(Stmt.Var stmt) {
        Object value = stmt.initializer() == null ? null : evaluate(stmt.initializer());
        environment.define(stmt.name().lexeme(), value);
        return null;
    }

    @Override
    public Void visitWhileStmt(Stmt.While stmt) {
        while (isTruthy(evaluate(stmt.condition()))) {
            execute(stmt.body());
        }
        return null;
    }

    @Override
    public Object visitAssignExpr(Expr.Assign expr) {
        var value = evaluate(expr.value());
        environment.assign(expr.name(), value);
        return value;
    }

    @Override
    public Object visitBinaryExpr(Expr.Binary expr) {
        var left = evaluate(expr.left());
        var right = evaluate(expr.right());

        switch (expr.operator().type()) {
            case MINUS, SLASH, STAR, GREATER, GREATER_EQUAL, LESS, LESS_EQUAL ->
                    checkNumberOperands(expr.operator(), left, right);
        }

        return switch (expr.operator().type()) {
            case MINUS -> (double) left - (double) right;
            case PLUS  -> {
                if (left instanceof Double leftD && right instanceof Double rightD) {
                    yield leftD + rightD;
                }

                if (left instanceof String leftStr && right instanceof String rightStr) {
                    yield leftStr + rightStr;
                }

                throw new RuntimeError(expr.operator(), "Operands must be two numbers or two strings.");
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
    public Object visitGroupingExpr(Expr.Grouping expr) {
        return evaluate(expr.expression());
    }

    @Override
    public Object visitLiteralExpr(Expr.Literal expr) {
        return expr.value();
    }

    @Override
    public Object visitLogicalExpr(Expr.Logical expr) {
        var leftVal = evaluate(expr.left());

        if (expr.operator().type() == TokenType.OR) {
            if (isTruthy(leftVal)) return leftVal;
        } else {
            if (!isTruthy(leftVal)) return leftVal;
        }

        return evaluate(expr.right());
    }

    @Override
    public Object visitUnaryExpr(Expr.Unary expr) {
        var right = evaluate(expr.right());

        return switch(expr.operator().type()) {
            case MINUS -> -asNumber(expr.operator(), right);
            case BANG -> !isTruthy(right);
            default -> null;
        };
    }

    @Override
    public Object visitVariableExpr(Expr.Variable expr) {
        return environment.get(expr.name());
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
