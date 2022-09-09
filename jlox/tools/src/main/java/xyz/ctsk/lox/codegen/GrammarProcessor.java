package xyz.ctsk.lox.codegen;


import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.RoundEnvironment;
import javax.annotation.processing.SupportedAnnotationTypes;
import javax.lang.model.element.Element;
import javax.lang.model.element.Name;
import javax.lang.model.element.PackageElement;
import javax.lang.model.element.TypeElement;
import javax.tools.Diagnostic;
import java.io.IOException;
import java.io.Writer;
import java.util.Set;

import static xyz.ctsk.lox.codegen.Grammar.Rule;
import static xyz.ctsk.lox.codegen.Grammar.Root;

@SupportedAnnotationTypes(
        "xyz.ctsk.lox.codegen.Grammar"
)
public class GrammarProcessor extends AbstractProcessor {
    private static final int INDENT = 4;

    private String getVisitorName(String className, String parentName) {
        return "visit%s%s".formatted(className, parentName);
    }

    @SuppressWarnings("SameParameterValue")
    private void writeVisitor(Rule[] rules, String parentName, Writer writer, int indent) throws IOException {
        writer.write("""
				interface Visitor<R> {
				""".indent(indent));

        for (var rule : rules) {
            var visitorName = getVisitorName(rule.head(), parentName);
            var parameter = "%s %s".formatted(rule.head(), parentName.toLowerCase());
            writer.write("""
					R %s(%s);
					""".formatted(visitorName, parameter).indent(indent + INDENT));
        }

        writer.write("""
				}
				
				""".indent(indent));
    }
    @SuppressWarnings("SameParameterValue")
    private void processRule(Rule rule, String parentBaseName, Writer writer, int indent) throws IOException {
        var fields = String.join(", ", rule.body());
        var recordName = rule.head();
        var visitorName = getVisitorName(recordName, parentBaseName);

        writer.write("""
    			
				record %s(%s) implements %s {
					@Override
					public <R> R accept(Visitor<R> visitor) {
						return visitor.%s(this);
					}
				}
				""".formatted(recordName, fields, parentBaseName, visitorName).indent(indent));
    }
    private void processRoot(Root root, Name packageName) {
        var baseName = root.name();
        var qualifiedName = "%s.%s".formatted(packageName, baseName);

        try {
            var file = processingEnv.getFiler().createSourceFile(qualifiedName);
            try (var writer = file.openWriter()) {
                writer.write("""
						package %s;
						
						import java.util.List;
						
						interface %s {
						""".formatted(packageName, baseName)
                );

                writeVisitor(root.rules(), baseName, writer, INDENT);

                writer.write("""
                        <R> R accept(Visitor<R> visitor);
                        
                        """.indent(INDENT)
                );

                for (var rule : root.rules()) {
                    processRule(rule, baseName, writer, INDENT);
                }

                writer.write("""
						}
						""");
            }
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    private void processGrammar(Element element) {
        if (element instanceof PackageElement packageElement) {
            for (var root : element.getAnnotation(Grammar.class).value()) {
                processRoot(root, packageElement.getQualifiedName());
            }
        } else {
            processingEnv.getMessager()
                    .printMessage(Diagnostic.Kind.ERROR, "@Grammar annotation can only be applied to a package");
        }
    }

    public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
        for (var annotation : annotations) {
            roundEnv.getElementsAnnotatedWith(annotation)
                    .forEach(this::processGrammar);
        }

        return true;
    }
}
