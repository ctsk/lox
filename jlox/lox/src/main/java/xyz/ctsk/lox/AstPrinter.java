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
        public String visitBinaryExpr(Expr.Binary binary) {
            var left = binary.left().accept(this);
            var op = binary.operator().lexeme();
            var right = binary.right().accept(this);

            return reverse ? wrap(left, right, op) : wrap(op, left, right);
        }

        @Override
        public String visitGroupingExpr(Expr.Grouping grouping) {
            var inner = grouping.expression().accept(this);
            return wrap(inner);
        }

        @Override
        public String visitLiteralExpr(Expr.Literal literal) {
            if (literal.value() == null) return "nil";
            return literal.value().toString();
        }

        @Override
        public String visitUnaryExpr(Expr.Unary unary) {
            var op = unary.operator().lexeme();
            var right = unary.right().accept(this);

            return reverse ? wrap(right, op) : wrap(op, right);
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
