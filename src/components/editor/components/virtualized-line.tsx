import { useRef, useState, useMemo } from 'react';
import clsx from 'clsx';
import { ViewportTokenizer } from '../services/viewport-tokenizer.ts';
import { CHAR_HEIGHT, LINE_PADDING_LEFT, LINE_PADDING_TOP } from '../constants.ts';
import { tokenStyles } from '../services/tokenizer.ts';
import { progressiveMacroTokenStyles } from '../services/macro-tokenizer-progressive.ts';
import { assemblyTokenStyles } from '../services/assembly-tokenizer.ts';
import { AssemblyHoverTooltip } from './assembly-hover-tooltip.tsx';

interface VirtualizedLineProps {
    tokens: any[];
    lineText: string;
    lineIndex: number;
    charWidth: number;
    isProgressiveMacro: boolean;
    isAssembly?: boolean;
    hasBreakpoint: boolean;
    isCurrentLine: boolean;
    isRunning: boolean;
    showDebug: boolean;
    onTokenClick?: (e: React.MouseEvent, token: any) => void;
    isMetaKeyHeld?: boolean;
    isShiftKeyHeld?: boolean;
    editorWidth: number;  // Width of the editor viewport
    editorScrollLeft?: number;  // Horizontal scroll position of the editor
}

export function VirtualizedLine({
    tokens,
    lineText,
    lineIndex,
    charWidth,
    isProgressiveMacro,
    isAssembly = false,
    hasBreakpoint,
    isCurrentLine,
    isRunning,
    showDebug,
    onTokenClick,
    isMetaKeyHeld,
    isShiftKeyHeld,
    editorWidth,
    editorScrollLeft = 0
}: VirtualizedLineProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const [localScrollLeft, setLocalScrollLeft] = useState(0);
    const [hoveredToken, setHoveredToken] = useState<{token: any, index: number} | null>(null);
    const [mousePosition, setMousePosition] = useState<{x: number, y: number}>({x: 0, y: 0});
    
    // Use editor scroll if available, otherwise use local scroll
    const scrollLeft = editorScrollLeft || localScrollLeft;
    
    const styles = isAssembly ? assemblyTokenStyles : (isProgressiveMacro ? progressiveMacroTokenStyles : tokenStyles);
    
    // Calculate if we should virtualize
    const shouldVirtualize = lineText.length > 1000;
    
    // Create viewport tokenizer using editor width
    const viewportTokenizer = useMemo(() => {
        return new ViewportTokenizer(charWidth, editorWidth, 0);
    }, [charWidth, editorWidth]);
    
    // Handle horizontal scrolling (only for individual line scrolling)
    const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
        if (!editorScrollLeft) {
            setLocalScrollLeft((e.target as HTMLDivElement).scrollLeft);
        }
    };
    
    // Get visible tokens
    const visibleTokens = useMemo(() => {
        // Update viewport position before filtering
        viewportTokenizer.updateViewport(editorWidth, scrollLeft);
        const filtered = viewportTokenizer.filterTokensForViewport(tokens, lineText);
        return filtered;
    }, [tokens, lineText, viewportTokenizer, scrollLeft, editorWidth]);
    
    // Calculate total line width
    const totalWidth = lineText.length * charWidth;
    
    if (!shouldVirtualize) {
        // For short lines, use the original rendering
        return (
            <>
                <div
                    ref={containerRef}
                    className={clsx(
                        "whitespace-pre pl-2 pr-4", {
                            "bg-zinc-900": showDebug && isCurrentLine && isRunning && !hasBreakpoint,
                            "bg-red-950": showDebug && hasBreakpoint
                        }
                    )}
                    style={{height: `${CHAR_HEIGHT}px`, lineHeight: `${CHAR_HEIGHT}px`}}
                >
                    {tokens.length === 0 ? (
                        <span>&nbsp;</span>
                    ) : (
                        tokens.map((token, tokenIndex) => (
                            <span
                                key={tokenIndex}
                                className={clsx(styles[token.type as keyof typeof styles] || '', {
                                    'cursor-pointer hover:underline': isMetaKeyHeld && ((token.type === 'macro_invocation' || token.type === 'hash_macro_invocation' || token.type === 'macro_name') && isProgressiveMacro || (token.type === 'label' || token.type === 'label_ref') && isAssembly),
                                    'cursor-pointer hover:bg-zinc-800 hover:rounded': isShiftKeyHeld && (token.type === 'macro_invocation' || token.type === 'hash_macro_invocation' || token.type === 'macro_name') && isProgressiveMacro
                                })}
                                onClick={(e) => onTokenClick?.(e, token)}
                                onMouseEnter={(e) => {
                                    if (isAssembly && isMetaKeyHeld && (token.type === 'instruction' || token.type === 'register' || token.type === 'directive')) {
                                        const rect = e.currentTarget.getBoundingClientRect();
                                        setHoveredToken({token, index: tokenIndex});
                                        setMousePosition({
                                            x: rect.left - (containerRef.current?.getBoundingClientRect().left || 0),
                                            y: rect.top - (containerRef.current?.getBoundingClientRect().top || 0)
                                        });
                                    }
                                }}
                                onMouseLeave={() => {
                                    if (hoveredToken?.index === tokenIndex) {
                                        setHoveredToken(null);
                                    }
                                }}
                            >
                                {token.value}
                            </span>
                        ))
                    )}
                </div>
                {isAssembly && hoveredToken && isMetaKeyHeld && (
                    <AssemblyHoverTooltip
                        token={hoveredToken.token}
                        x={mousePosition.x}
                        y={mousePosition.y}
                        visible={true}
                    />
                )}
            </>
        );
    }
    
    // For long lines, use virtualized rendering
    // If we have editor scroll, don't add local scroll container
    if (editorScrollLeft !== undefined) {
        return (
            <>
                <div
                    ref={containerRef}
                    className={clsx(
                        "whitespace-pre pl-2 pr-4", {
                            "bg-zinc-900": showDebug && isCurrentLine && isRunning && !hasBreakpoint,
                            "bg-red-950": showDebug && hasBreakpoint
                        }
                    )}
                    style={{
                        height: `${CHAR_HEIGHT}px`,
                        lineHeight: `${CHAR_HEIGHT}px`,
                        width: `${totalWidth}px`
                    }}
                >
                <div className="relative">
                    {visibleTokens.map((token, tokenIndex) => {
                        if (token.type === 'truncation-indicator') {
                            // Don't render truncation indicators when using editor scroll
                            return null;
                        }
                        
                        return (
                            <span
                                key={tokenIndex}
                                className={clsx(styles[token.type as keyof typeof styles] || '', {
                                    'cursor-pointer hover:underline': isMetaKeyHeld && ((token.type === 'macro_invocation' || token.type === 'hash_macro_invocation' || token.type === 'macro_name') && isProgressiveMacro || (token.type === 'label' || token.type === 'label_ref') && isAssembly),
                                    'cursor-pointer hover:bg-zinc-800 hover:px-1 hover:rounded': isShiftKeyHeld && (token.type === 'macro_invocation' || token.type === 'hash_macro_invocation' || token.type === 'macro_name') && isProgressiveMacro
                                })}
                                style={{
                                    position: 'absolute',
                                    left: `${token.originalStart * charWidth}px`
                                }}
                                onClick={(e) => onTokenClick?.(e, token)}
                                onMouseEnter={(e) => {
                                    if (isAssembly && isMetaKeyHeld && (token.type === 'instruction' || token.type === 'register' || token.type === 'directive')) {
                                        const rect = e.currentTarget.getBoundingClientRect();
                                        setHoveredToken({token, index: tokenIndex});
                                        setMousePosition({
                                            x: rect.left - (containerRef.current?.getBoundingClientRect().left || 0),
                                            y: rect.top - (containerRef.current?.getBoundingClientRect().top || 0)
                                        });
                                    }
                                }}
                                onMouseLeave={() => {
                                    if (hoveredToken?.index === tokenIndex) {
                                        setHoveredToken(null);
                                    }
                                }}
                            >
                                {token.value}
                            </span>
                        );
                    })}
                </div>
            </div>
            {isAssembly && hoveredToken && isMetaKeyHeld && (
                <AssemblyHoverTooltip
                    token={hoveredToken.token}
                    x={mousePosition.x}
                    y={mousePosition.y}
                    visible={true}
                />
            )}
        </>
        );
    }
    
    // For individual line scrolling (fallback)
    return (
        <>
            <div
                ref={containerRef}
                className={clsx(
                "overflow-x-auto whitespace-pre", {
                    "bg-zinc-900": showDebug && isCurrentLine && isRunning && !hasBreakpoint,
                    "bg-red-950": showDebug && hasBreakpoint
                }
            )}
            style={{height: `${CHAR_HEIGHT}px`}}
            onScroll={handleScroll}
        >
            <div 
                className="relative pl-2 pr-4"
                style={{
                    width: `${totalWidth}px`,
                    height: `${CHAR_HEIGHT}px`,
                    lineHeight: `${CHAR_HEIGHT}px`
                }}
            >
                <div 
                    className="absolute top-0 left-0"
                    style={{
                        transform: `translateX(${-scrollLeft}px)`,
                        paddingLeft: '8px' // pl-2
                    }}
                >
                    {visibleTokens.map((token, tokenIndex) => {
                        if (token.type === 'truncation-indicator') {
                            const position = token.originalStart === 0 ? scrollLeft - charWidth : scrollLeft + editorWidth;
                            return (
                                <span
                                    key={tokenIndex}
                                    className="text-zinc-500 opacity-50"
                                    style={{
                                        position: 'absolute',
                                        left: `${position}px`
                                    }}
                                >
                                    {token.value}
                                </span>
                            );
                        }
                        
                        return (
                            <span
                                key={tokenIndex}
                                className={clsx(styles[token.type as keyof typeof styles] || '', {
                                    'cursor-pointer hover:underline': isMetaKeyHeld && ((token.type === 'macro_invocation' || token.type === 'hash_macro_invocation' || token.type === 'macro_name') && isProgressiveMacro || (token.type === 'label' || token.type === 'label_ref') && isAssembly),
                                    'cursor-pointer hover:bg-zinc-800 hover:px-1 hover:rounded': isShiftKeyHeld && (token.type === 'macro_invocation' || token.type === 'hash_macro_invocation' || token.type === 'macro_name') && isProgressiveMacro
                                })}
                                style={{
                                    position: 'absolute',
                                    left: `${token.originalStart * charWidth}px`
                                }}
                                onClick={(e) => onTokenClick?.(e, token)}
                                onMouseEnter={(e) => {
                                    if (isAssembly && isMetaKeyHeld && (token.type === 'instruction' || token.type === 'register' || token.type === 'directive')) {
                                        const rect = e.currentTarget.getBoundingClientRect();
                                        setHoveredToken({token, index: tokenIndex});
                                        setMousePosition({
                                            x: rect.left - (containerRef.current?.getBoundingClientRect().left || 0),
                                            y: rect.top - (containerRef.current?.getBoundingClientRect().top || 0)
                                        });
                                    }
                                }}
                                onMouseLeave={() => {
                                    if (hoveredToken?.index === tokenIndex) {
                                        setHoveredToken(null);
                                    }
                                }}
                            >
                                {token.value}
                            </span>
                        );
                    })}
                </div>
            </div>
        </div>
        {isAssembly && hoveredToken && isMetaKeyHeld && (
            <AssemblyHoverTooltip
                token={hoveredToken.token}
                x={mousePosition.x}
                y={mousePosition.y}
                visible={true}
            />
        )}
    </>
    );
}