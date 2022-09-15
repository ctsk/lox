package xyz.ctsk.lox;

import com.oracle.truffle.api.frame.FrameDescriptor;
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
        var parseResult = LoxParser.parseLox(program);
        var root = new LoxRootNode(parseResult.rootNode(), parseResult.frame());
        var callTarget = root.getCallTarget();
        System.out.println(callTarget.call());
    }

    public static void main(String[] args) throws IOException {
        repl();
    }
}
