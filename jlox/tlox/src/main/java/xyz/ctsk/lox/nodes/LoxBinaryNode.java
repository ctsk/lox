package xyz.ctsk.lox.nodes;

import com.oracle.truffle.api.dsl.NodeChild;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

@NodeChild("leftValue")
@NodeChild("rightValue")
public abstract class LoxBinaryNode extends LoxExpressionNode {
}
