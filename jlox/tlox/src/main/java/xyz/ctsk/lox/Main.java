package xyz.ctsk.lox;

import xyz.ctsk.lox.nodes.LoxRootNode;
import xyz.ctsk.lox.parser.LoxParser;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

public class Main {

    public static void repl() throws IOException {
        var input = new InputStreamReader(System.in);
        var reader = new BufferedReader(input);

        while (true) {
            System.out.print(" >> ");
            var line = reader.readLine();
            if (line == null) break;
            run(line);
        }
    }

    public static void run(String program) {
        var parsed = LoxParser.parseLox(program);

        if (parsed instanceof LoxParser.ExpressionContext expr) {
            var root = new LoxRootNode(expr.result);
            var callTarget = root.getCallTarget();
            System.out.println(callTarget.call());
        }
    }

    public static void main(String[] args) throws IOException {
        repl();
    }
}
