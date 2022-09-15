package xyz.ctsk.lox.parser;

import com.oracle.truffle.api.frame.FrameDescriptor;
import xyz.ctsk.lox.nodes.LoxExpressionNode;

public record ParseResult(LoxExpressionNode rootNode, FrameDescriptor frame) { }
