package xyz.ctsk.lox;

public class AstPrinter {
    public static String polish(Expr expr, boolean parentheses) {
        return expr.accept(new Polish(parentheses, false));
    }

    public static String reverse_polish(Expr expr, boolean parentheses) {
        return expr.accept(new Polish(parentheses, true));
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
}
