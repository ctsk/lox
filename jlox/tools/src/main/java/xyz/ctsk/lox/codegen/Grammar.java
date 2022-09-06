package xyz.ctsk.lox.codegen;

import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

@Retention(RetentionPolicy.SOURCE)
public @interface Grammar {
    Root[] value();

    @interface Root {
        String name();
        Rule[] rules();
    }
    @interface Rule {
        String head();
        String[] body();
    }
}

