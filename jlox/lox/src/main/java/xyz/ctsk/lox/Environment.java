package xyz.ctsk.lox;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

public class Environment {
    final Environment enclosing;
    private final Map<String, Object> values = new HashMap<>();

    Environment() {
        enclosing = null;
    }

    Environment(Environment enclosing) {
        this.enclosing = enclosing;
    }

    void define(String name, Object value) {
        values.put(name, value);
    }

    private Environment ancestor(int distance) {
        Environment environment = this;

        for (int i = 0; i < distance; i++) {
            environment = environment.enclosing;
        }

        return environment;
    }

    void assign(Token name, Object value) {
        if (values.containsKey(name.lexeme())) {
            values.put(name.lexeme(), value);
        } else if (enclosing != null) {
            enclosing.assign(name, value);
        } else {
            throw new RuntimeError(name, "Undefined variable '%s'.".formatted(name.lexeme()));
        }
    }

    void assignAt(int distance, Token name, Object value) {
        ancestor(distance).values.put(name.lexeme(), value);
    }

    Object get(Token name) {
        if (values.containsKey(name.lexeme())) {
            return values.get(name.lexeme());
        } else if (enclosing != null) {
            return enclosing.get(name);
        } else {
            var message = "Undefined variable '%s'.".formatted(name.lexeme());
            throw new RuntimeError(name, message);
        }
    }

    Object getAt(int distance, String name) {
        return ancestor(distance).values.get(name);
    }

}
