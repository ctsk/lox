package xyz.ctsk.lox.nodes;

import com.oracle.truffle.api.dsl.NodeChild;

@NodeChild("value")
public abstract class LoxUnaryNode extends LoxExpressionNode {
}
