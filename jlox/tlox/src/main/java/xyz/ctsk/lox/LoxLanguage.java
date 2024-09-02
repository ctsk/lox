package xyz.ctsk.lox;

import com.oracle.truffle.api.CallTarget;
import com.oracle.truffle.api.TruffleLanguage;
import xyz.ctsk.lox.nodes.LoxExpressionNode;
import xyz.ctsk.lox.nodes.LoxNode;
import xyz.ctsk.lox.nodes.LoxRootNode;
import xyz.ctsk.lox.parser.LoxParser;

@TruffleLanguage.Registration(id = LoxLanguage.ID, name = LoxLanguage.NAME)
public class LoxLanguage extends TruffleLanguage<Void> {
    public static final String ID = "lox";
    public static final String NAME = "Lox";

    @Override
    protected CallTarget parse(ParsingRequest request) throws Exception {
        LoxNode node = LoxParser.parseLox(request.getSource().getReader());
        var rootNode = new LoxRootNode((LoxExpressionNode) node);
        return rootNode.getCallTarget();
    }

    @Override
    protected Void createContext(Env env) {
        return null;
    }
}
