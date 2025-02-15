package xyz.ctsk.lox;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;

public class Lox {
    private static final Interpreter interpreter = new Interpreter();

    private static boolean hadError = false;
    private static boolean hadRuntimeError = false;

    private static void report(int line, String where, String message) {
         System.err.printf("[line %d] Error%s: %s%n", line, where, message);
         hadError = true;
    }

    static void error(int line, String message) {
        report(line, "", message);
    }

    static void error(Token token, String message) {
        if (token.type() == TokenType.EOF) {
            report(token.line(), " at end", message);
        } else {
            report(token.line()," at '%s'".formatted(token.lexeme()), message);
        }
    }

    static void runtimeError(RuntimeError error) {
        System.err.printf("%s%n[line %d]%n", error.getMessage(), error.token.line());
        hadRuntimeError = true;
    }

    private static void run(String source) {
        var scanner = new Scanner(source);
        var tokens = scanner.scanTokens();

        Parser parser = new Parser(tokens);
        List<Stmt> statements = parser.parse();

        if (hadError) return;

        var resolver = new Resolver(interpreter);
        resolver.resolve(statements);

        if (hadError) return;

        interpreter.interpret(statements);
    }

    private static void runPrompt() throws IOException {
        var input = new InputStreamReader(System.in);
        var reader = new BufferedReader(input);

        while (true) {
            System.out.print("> ");
            String line = reader.readLine();
            if (line == null) break;
            run(line);
            hadError = false;
        }
    }

    protected static int runFile(Path path) throws IOException {
        byte[] bytes = Files.readAllBytes(path);
        run(new String(bytes, Charset.defaultCharset()));

        if (hadError) return 65;
        if (hadRuntimeError) return 70;
        return 0;
    }

    protected static int runFile(String path) throws IOException {
        return runFile(Paths.get(path));
    }

    private static void printUsage() {
        System.out.println("Usage: jlox [script]");
    }

    public static void main(String[] args) throws IOException {
        if (args.length == 0) {
            runPrompt();
        } else if (args.length == 1) {
            System.exit(runFile(args[0]));
        } else {
            printUsage();
        }
    }
}
