import { useMemo } from "react";
import { CHAR_HEIGHT, LINE_PADDING_LEFT, LINE_PADDING_TOP } from "../constants.ts";

interface UnusedMacroHighlightsProps {
    unusedMacros: Set<string>;
    tokenizedLines: any[][];
    charWidth: number;
}

export function UnusedMacroHighlights({ 
    unusedMacros, 
    tokenizedLines, 
    charWidth 
}: UnusedMacroHighlightsProps) {
    // Find positions of all unused macro definitions
    const highlights = useMemo(() => {
        const positions: Array<{
            line: number;
            startColumn: number;
            endColumn: number;
        }> = [];
        
        tokenizedLines.forEach((tokens, lineIndex) => {
            tokens.forEach((token) => {
                if (token.type === 'macro_name' && unusedMacros.has(token.value)) {
                    positions.push({
                        line: lineIndex,
                        startColumn: token.start,
                        endColumn: token.end
                    });
                }
            });
        });
        
        return positions;
    }, [unusedMacros, tokenizedLines]);
    
    if (highlights.length === 0) return null;
    
    return (
        <>
            {highlights.map((highlight, index) => {
                const x = highlight.startColumn * charWidth + LINE_PADDING_LEFT;
                const y = highlight.line * CHAR_HEIGHT + LINE_PADDING_TOP;
                const width = (highlight.endColumn - highlight.startColumn) * charWidth;
                
                return (
                    <div key={index}>
                        {/* Dim overlay to make unused macros appear faded */}
                        <div
                            className="absolute pointer-events-none"
                            style={{
                                left: `${x}px`,
                                top: `${y}px`,
                                width: `${width}px`,
                                height: `${CHAR_HEIGHT}px`,
                                backgroundColor: 'rgba(0, 0, 0, 0.5)',
                                mixBlendMode: 'multiply'
                            }}
                        />
                        {/* Subtle dotted underline */}
                        <div
                            className="absolute pointer-events-none opacity-30"
                            style={{
                                left: `${x}px`,
                                top: `${y + CHAR_HEIGHT - 1}px`,
                                width: `${width}px`,
                                height: '1px',
                                background: 'repeating-linear-gradient(to right, rgb(156 163 175) 0px, rgb(156 163 175) 1px, transparent 1px, transparent 3px)'
                            }}
                        />
                    </div>
                );
            })}
        </>
    );
}