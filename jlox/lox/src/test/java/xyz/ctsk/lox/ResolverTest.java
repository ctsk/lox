package xyz.ctsk.lox;

import org.junit.jupiter.api.Test;

import java.io.IOException;

public class ResolverTest {
    @Test
    void forLoopBug() throws IOException {
        var canary = this.getClass().getResource("ResolverBug.lox").getPath();
        Lox.runFile(canary);
    }
}
