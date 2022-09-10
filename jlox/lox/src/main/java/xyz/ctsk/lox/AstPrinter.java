package xyz.ctsk.lox;

import java.util.stream.Collectors;

public class AstPrinter {
    public static String polish(Expr expr, boolean parentheses) {
        return expr.accept(new Polish(parentheses, false));
    }

    public static String reverse_polish(Expr expr, boolean parentheses) {
        return expr.accept(new Polish(parentheses, true));
    }

    public static String pretty(Expr expr) {
        return expr.accept(new Pretty());
    }

    private record Polish(boolean parentheses, boolean reverse) implements Expr.Visitor<String> {
        @Override
        public String visitAssignExpr(Expr.Assign expr) {
            var target = expr.name().lexeme();
            var value = expr.value().accept(this);

            return reverse ? wrap(target, value, "=") : wrap("=", target, value);
        }

        @Override
        public String visitBinaryExpr(Expr.Binary expr) {
            var left = expr.left().accept(this);
            var op = expr.operator().lexeme();
            var right = expr.right().accept(this);

            return reverse ? wrap(left, right, op) : wrap(op, left, right);
        }

        @Override
        public String visitCallExpr(Expr.Call expr) {
            var fun = expr.callee().accept(this);
            var args = expr.arguments().stream()
                    .map(c -> c.accept(this))
                    .collect(Collectors.joining(" "));

            return reverse ? wrap(fun, args, "call") : wrap("call", fun, args);
        }

        @Override
        public String visitGroupingExpr(Expr.Grouping expr) {
            var inner = expr.expression().accept(this);
            return wrap(inner);
        }

        @Override
        public String visitLiteralExpr(Expr.Literal expr) {
            if (expr.value() == null) return "nil";
            return expr.value().toString();
        }

        @Override
        public String visitLogicalExpr(Expr.Logical expr) {
            var left = expr.left().accept(this);
            var op = expr.operator().lexeme();
            var right = expr.right().accept(this);

            return reverse ? wrap(left, right, op) : wrap(op, left, right);
        }

        @Override
        public String visitUnaryExpr(Expr.Unary expr) {
            var op = expr.operator().lexeme();
            var right = expr.right().accept(this);

            return reverse ? wrap(right, op) : wrap(op, right);
        }

        @Override
        public String visitVariableExpr(Expr.Variable expr) {
            return expr.name().lexeme();
        }

        public String wrap(String... inner) {
            var inners = String.join(" ", inner);

            if (parentheses) {
                return "(" + inners + ")";
            } else {
                return inners;
            }
        }
    }

    private static class Pretty implements Expr.Visitor<String> {
        @Override
        public String visitAssignExpr(Expr.Assign expr) {
            return String.join(" ", expr.name().lexeme(), "=", expr.value().accept(this));
        }

        @Override
        public String visitBinaryExpr(Expr.Binary expr) {
            return String.join(" ",
                    expr.left().accept(this),
                    expr.operator().lexeme(),
                    expr.right().accept(this));
        }

        @Override
        public String visitCallExpr(Expr.Call expr) {
            var fun = expr.callee().accept(this);
            var args = expr.arguments().stream()
                    .map(c -> c.accept(this))
                    .collect(Collectors.joining(", "));
            return fun + "(" + args + ")";
        }

        @Override
        public String visitGroupingExpr(Expr.Grouping expr) {
            return '(' + expr.expression().accept(this) + ')';
        }

        @Override
        public String visitLiteralExpr(Expr.Literal expr) {
            return Interpreter.stringify(expr.value());
        }

        @Override
        public String visitLogicalExpr(Expr.Logical expr) {
            return String.join(" ",
                    expr.left().accept(this),
                    expr.operator().lexeme(),
                    expr.right().accept(this));
        }

        @Override
        public String visitUnaryExpr(Expr.Unary expr) {
            return expr.operator().lexeme() + expr.right().accept(this);
        }

        @Override
        public String visitVariableExpr(Expr.Variable expr) {
            return expr.name().lexeme();
        }
    }
}
