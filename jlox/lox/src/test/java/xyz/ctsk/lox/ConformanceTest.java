package xyz.ctsk.lox;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

import java.io.IOException;
import java.io.PrintStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.stream.Stream;
import java.util.stream.StreamSupport;


public class ConformanceTest {
    private static final Path LOX_ROOT = Path.of("src/test/resources");

    static Stream<String> loxPaths() throws IOException {
        return Files.walk(LOX_ROOT)
                .filter(Files::isRegularFile)
                .map(Path::toString);
    }

    @ParameterizedTest(name = "{index}")
    @MethodSource("loxPaths")
    public void run(String path) throws IOException {
        System.out.println(path);
        Lox.runFile(path);
    }

    @BeforeEach
    public void setUpStreams() {
        System.setOut(new PrintStream(System.out, true));
    }
}
