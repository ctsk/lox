package xyz.ctsk.lox;


import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Stack;

public class Resolver implements Expr.Visitor<Void>, Stmt.Visitor<Void> {
    private final Interpreter interpreter;
    private final Stack<Map<String, Boolean>> scopes = new Stack<>();
    private FunctionType currentFuntion = FunctionType.NONE;

    private enum ClassType { NONE, CLASS }

    private ClassType currentClass = ClassType.NONE;


    Resolver(Interpreter interpreter) {
        this.interpreter = interpreter;
    }

    private void resolve(Stmt stmt) {
        stmt.accept(this);
    }

    void resolve(List<Stmt> statements) {
        statements.forEach(this::resolve);
    }

    private void resolve(Expr expr) {
        expr.accept(this);
    }


    private void beginScope() {
        scopes.push(new HashMap<>());
    }

    private void endScope() {
        scopes.pop();
    }

    private void declare(Token name) {
        if (scopes.isEmpty()) return;
        var scope = scopes.peek();
        if (scope.containsKey(name.lexeme())) {
            Lox.error(name, "Already a variable with this name in this scope.");
        }
        scope.put(name.lexeme(), false);
    }

    private void define(Token name) {
        if (scopes.isEmpty()) return;
        scopes.peek().put(name.lexeme(), true);
    }

    private void resolveFunction(Stmt.Function function, FunctionType functionType) {
        var enclosingFunction = currentFuntion;
        currentFuntion = functionType;
        beginScope();
        for (var param : function.params()) {
            declare(param);
            define(param);
        }
        resolve(function.body());
        endScope();
        currentFuntion = enclosingFunction;
    }


    private void resolveLocal(Expr expr, Token name) {
        for (int i = scopes.size() - 1; i >= 0; i--) {
            if (scopes.get(i).containsKey(name.lexeme())) {
                interpreter.resolve(expr, scopes.size() - 1 - i);
                return;
            }
        }
    }

    @Override
    public Void visitBlockStmt(Stmt.Block stmt) {
        beginScope();
        resolve(stmt.statements());
        endScope();
        return null;
    }

    @Override
    public Void visitClassStmt(Stmt.Class stmt) {
        var enclosing = currentClass;
        currentClass = ClassType.CLASS;
        declare(stmt.name());
        define(stmt.name());
        beginScope();
        scopes.peek().put("this", true);

        for (var method : stmt.methods()) {
            FunctionType declaration = FunctionType.METHOD;

            if (method.name().lexeme().equals("init")) {
                declaration = FunctionType.INITIALIZER;
            }

            resolveFunction(method, declaration);
        }

        endScope();

        currentClass = enclosing;
        return null;
    }


    @Override
    public Void visitExpressionStmt(Stmt.Expression stmt) {
        resolve(stmt.expression());
        return null;
    }

    @Override
    public Void visitFunctionStmt(Stmt.Function stmt) {
        declare(stmt.name());
        define(stmt.name());
        resolveFunction(stmt, FunctionType.FUNCTION);
        return null;
    }

    @Override
    public Void visitIfStmt(Stmt.If stmt) {
        resolve(stmt.condition());
        resolve(stmt.thenBranch());
        if (stmt.elseBranch() != null) resolve(stmt.elseBranch());
        return null;
    }

    @Override
    public Void visitPrintStmt(Stmt.Print stmt) {
        resolve(stmt.expression());
        return null;
    }

    @Override
    public Void visitReturnStmt(Stmt.Return stmt) {
        if (currentFuntion == FunctionType.NONE) {
            Lox.error(stmt.keyword(), "Can't return fom top-level code.");
        }

        if (stmt.value() != null) {
            if (currentFuntion == FunctionType.INITIALIZER) {
                Lox.error(stmt.keyword(), "Can't return a value from an initializer.");
            }
            resolve(stmt.value());
        }
        return null;
    }

    @Override
    public Void visitVarStmt(Stmt.Var stmt) {
        declare(stmt.name());
        if (stmt.initializer() != null) {
            resolve(stmt.initializer());
        }
        define(stmt.name());
        return null;
    }

    @Override
    public Void visitWhileStmt(Stmt.While stmt) {
        resolve(stmt.condition());
        resolve(stmt.body());
        return null;
    }


    @Override
    public Void visitAssignExpr(Expr.Assign expr) {
        resolve(expr.value());
        resolveLocal(expr, expr.name());
        return null;
    }

    @Override
    public Void visitBinaryExpr(Expr.Binary expr) {
        resolve(expr.left());
        resolve(expr.right());
        return null;
    }

    @Override
    public Void visitGetExpr(Expr.Get expr) {
        resolve(expr.object());
        return null;
    }

    @Override
    public Void visitCallExpr(Expr.Call expr) {
        resolve(expr.callee());
        expr.arguments().forEach(this::resolve);
        return null;
    }

    @Override
    public Void visitGroupingExpr(Expr.Grouping expr) {
        resolve(expr.expression());
        return null;
    }

    @Override
    public Void visitLiteralExpr(Expr.Literal expr) {
        return null;
    }

    @Override
    public Void visitLogicalExpr(Expr.Logical expr) {
        resolve(expr.left());
        resolve(expr.right());
        return null;
    }

    @Override
    public Void visitSetExpr(Expr.Set expr) {
        resolve(expr.value());
        resolve(expr.object());
        return null;
    }

    @Override
    public Void visitThisExpr(Expr.This expr) {
        if (currentClass == ClassType.NONE) {
            Lox.error(expr.keyword(), "Can't use 'this' outside of a class.");
            return null;
        }

        resolveLocal(expr, expr.keyword());
        return  null;
    }

    @Override
    public Void visitUnaryExpr(Expr.Unary expr) {
        resolve(expr.right());
        return null;
    }

    @Override
    public Void visitVariableExpr(Expr.Variable expr) {
        if (!scopes.isEmpty() && scopes.peek().get(expr.name().lexeme()) == Boolean.FALSE) {
            Lox.error(expr.name(), "Can't read local variable in its own initializer.");
        }
        resolveLocal(expr, expr.name());
        return null;
    }
}
