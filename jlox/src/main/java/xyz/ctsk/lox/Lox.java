package xyz.ctsk.lox;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Paths;

public class Lox {
    private static boolean hadError = false;
     private static void run(String source) {
        var scanner = new Scanner(source);
        scanner.scanTokens().forEach(System.out::println);
    }

    static void error(int line, String message) {
         report(line, "", message);
    }

    private static void report(int line, String where, String message) {
         System.err.printf("[line %d] Error %s: %s%n", line, where, message);
         hadError = true;
    }


    private static void runPrompt() throws IOException {
        var input = new InputStreamReader(System.in);
        var reader = new BufferedReader(input);

        while (true) {
            System.out.println("> ");
            String line = reader.readLine();
            if (line == null) break;
            run(line);
            hadError = false;
        }
    };

    private static void runFile(String path) throws IOException {
        byte[] bytes = Files.readAllBytes(Paths.get(path));
        run(new String(bytes, Charset.defaultCharset()));

        if (hadError) {
            System.exit(65);
        }
    }

    private static void printUsage() {
        System.out.println("Usage: jlox [script]");
    }

    public static void main(String[] args) throws IOException {
        if (args.length == 0) {
            runPrompt();
        } else if (args.length == 1) {
            runFile(args[0]);
        } else {
            printUsage();
        }
    }
}
