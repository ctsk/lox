1. Java (OpenJDK)
  -> No Flex/Yacc used
  -> Files:
  	Scanner: [jdk/src/jdk.compiler/share/classes/com/sun/tools/javac/parser/Scanner.java]
	Parser:  [jdk/src/jdk.compiler/share/classes/com/sun/tools/javac/parser/JavacParser.java]
  	[https://github.com/openjdk/jdk/blob/1f484dae4efaa60cf18a3d4df947c05f1497bd5b]

2. Reasons not to JIT
  - Short running programs => Cost of compilation higher than gained speed
  - Little repeat execution of code
  - Platform-independence

3. Why do Lisps contain interpreters?
  - Better interactivity
  - Evaluation of Macros
