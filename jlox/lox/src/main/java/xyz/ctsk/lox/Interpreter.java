package xyz.ctsk.lox;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class Interpreter implements Expr.Visitor<Object>, Stmt.Visitor<Void> {
    final Environment globals = new Environment();
    private Environment environment = globals;
    private final Map<Expr, Integer> locals = new HashMap<>();


    Interpreter() {
        globals.define("clock", new LoxCallable() {
            @Override
            public int arity() {
                return 0;
            }

            @Override
            public Object call(Interpreter interpreter, List<Object> arguments) {
                return (double) System.currentTimeMillis() / 1000.0;
            }

            @Override
            public String toString() {
                return "<native fn>";
            }
        });
    }


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

    void executeBlock(List<Stmt> statements, Environment environment) {
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

    void resolve(Expr expr, int depth) {
        locals.put(expr, depth);
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
    public Void visitClassStmt(Stmt.Class stmt) {
        Object superclass = null;
        if (stmt.superclass() != null) {
            superclass = evaluate(stmt.superclass());
            if (!(superclass instanceof LoxClass)) {
                throw new RuntimeError(stmt.superclass().name(), "Superclass must be a class.");
            }
        }

        environment.define(stmt.name().lexeme(), null);

        if (stmt.superclass() != null) {
            environment = new Environment(environment);
            environment.define("super", superclass);
        }

        Map<String, LoxFunction> methods = new HashMap<>();
        for (var method : stmt.methods()) {
            var function = new LoxFunction(method, environment, method.name().lexeme().equals("init"));
            methods.put(method.name().lexeme(), function);
        }

        LoxClass clazz = new LoxClass(stmt.name().lexeme(), (LoxClass) superclass, methods);

        if (superclass != null) {
            environment = environment.enclosing;
        }

        environment.assign(stmt.name(), clazz);
        return null;
    }

    @Override
    public Void visitExpressionStmt(Stmt.Expression stmt) {
        evaluate(stmt.expression());
        return null;
    }

    @Override
    public Void visitFunctionStmt(Stmt.Function stmt) {
        environment.define(stmt.name().lexeme(), new LoxFunction(stmt, environment, false));
        return null;
    }

    @Override
    public Void visitIfStmt(Stmt.If stmt) {
        if (isTruthy(evaluate(stmt.condition()))) {
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
    public Void visitReturnStmt(Stmt.Return stmt) {
        var value = stmt.value() == null ? null : evaluate(stmt.value());
        throw new Return(value);
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

        Integer distance = locals.get(expr);
        if (distance != null) {
            environment.assignAt(distance, expr.name(), value);
        } else {
            globals.assign(expr.name(), value);
        }

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
    public Object visitGetExpr(Expr.Get expr) {
        var object = evaluate(expr.object());
        if (object instanceof LoxInstance loxInstance) {
            return loxInstance.get(expr.name());
        }
        throw new RuntimeError(expr.name(), "Only instances have properties.");
    }

    @Override
    public Object visitCallExpr(Expr.Call expr) {
        var callee = evaluate(expr.callee());
        var arguments = expr.arguments().stream()
                .map(this::evaluate)
                .toList();

        if (callee instanceof LoxCallable function) {
            if (arguments.size() != function.arity()) {
                var msg = "Expected %d arguments but got %d.".formatted(function.arity(), arguments.size());
                throw new RuntimeError(expr.paren(), msg);
            }

            return function.call(this, arguments);
        } else {
            throw new RuntimeError(expr.paren(), "Can only call functions and classes.");
        }
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
    public Object visitSetExpr(Expr.Set expr) {
        var object = evaluate(expr.object());
        if (object instanceof LoxInstance instance) {
            var value = evaluate(expr.value());
            instance.set(expr.name(), value);
            return value;
        } else {
            throw new RuntimeError(expr.name(), "Only instances have fields.");
        }
    }

    @Override
    public Object visitSuperExpr(Expr.Super expr) {
        int distance = locals.get(expr);
        LoxClass superclass = (LoxClass) environment.getAt(distance, "super");
        LoxInstance object = (LoxInstance) environment.getAt(distance - 1, "this");
        LoxFunction method = superclass.findMethod(expr.method().lexeme());

        if (method == null) {
            throw new RuntimeError(expr.method(),
                    "Undefined property '" + expr.method().lexeme() + "'.");
        }

        return method.bind(object);
    }

    @Override
    public Object visitThisExpr(Expr.This expr) {
        return lookupVariable(expr.keyword(), expr);
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
        return lookupVariable(expr.name(), expr);
    }

    private Object lookupVariable(Token name, Expr expr) {
        var distance = locals.get(expr);
        if (distance != null) {
            return environment.getAt(distance, name.lexeme());
        } else {
            return globals.get(name);
        }
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
        throw new RuntimeError(operator, "Operands must be numbers.");
    }
}
