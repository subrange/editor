// Viewport-based token rendering for extremely long lines
interface ViewportRange {
    startColumn: number;
    endColumn: number;
}

export interface ViewportToken {
    type: string;
    value: string;
    start: number;
    end: number;
    originalStart: number; // Original position in the full line
    originalEnd: number;
}

export class ViewportTokenizer {
    private charWidth: number;
    private containerWidth: number;
    private scrollLeft: number;
    
    constructor(charWidth: number, containerWidth: number, scrollLeft: number = 0) {
        this.charWidth = charWidth;
        this.containerWidth = containerWidth;
        this.scrollLeft = scrollLeft;
    }
    
    updateViewport(containerWidth: number, scrollLeft: number) {
        this.containerWidth = containerWidth;
        this.scrollLeft = scrollLeft;
    }
    
    private getViewportRange(): ViewportRange {
        // Calculate visible character range with some buffer for smooth scrolling
        const buffer = 50; // Extra characters to render on each side
        const startColumn = Math.max(0, Math.floor(this.scrollLeft / this.charWidth) - buffer);
        const visibleChars = Math.ceil(this.containerWidth / this.charWidth);
        const endColumn = startColumn + visibleChars + (buffer * 2);
        
        return { startColumn, endColumn };
    }
    
    // Filter and adjust tokens to only include visible ones
    filterTokensForViewport(tokens: any[], lineText: string): ViewportToken[] {
        const { startColumn, endColumn } = this.getViewportRange();
        const visibleTokens: ViewportToken[] = [];
        
        // If line is short enough, render all tokens
        if (lineText.length <= 1000) {
            return tokens.map(t => ({
                ...t,
                originalStart: t.start,
                originalEnd: t.end
            }));
        }
        
        
        for (const token of tokens) {
            // Skip tokens completely outside viewport
            if (token.end <= startColumn || token.start >= endColumn) {
                continue;
            }
            
            // Token is at least partially visible
            let visibleToken: ViewportToken;
            
            if (token.start >= startColumn && token.end <= endColumn) {
                // Token is completely visible
                visibleToken = {
                    ...token,
                    originalStart: token.start,
                    originalEnd: token.end,
                    start: token.start - startColumn,
                    end: token.end - startColumn
                };
            } else if (token.start < startColumn && token.end > endColumn) {
                // Token spans entire viewport
                visibleToken = {
                    ...token,
                    value: token.value.slice(startColumn - token.start, endColumn - token.start),
                    originalStart: token.start,
                    originalEnd: token.end,
                    start: 0,
                    end: endColumn - startColumn
                };
            } else if (token.start < startColumn) {
                // Token starts before viewport
                visibleToken = {
                    ...token,
                    value: token.value.slice(startColumn - token.start),
                    originalStart: token.start,
                    originalEnd: token.end,
                    start: 0,
                    end: token.end - startColumn
                };
            } else {
                // Token ends after viewport
                visibleToken = {
                    ...token,
                    value: token.value.slice(0, endColumn - token.start),
                    originalStart: token.start,
                    originalEnd: token.end,
                    start: token.start - startColumn,
                    end: endColumn - startColumn
                };
            }
            
            visibleTokens.push(visibleToken);
        }
        
        // Add indicators for truncated content
        if (startColumn > 0) {
            visibleTokens.unshift({
                type: 'truncation-indicator',
                value: '◀',
                start: -1,
                end: 0,
                originalStart: 0,
                originalEnd: 0
            });
        }
        
        if (endColumn < lineText.length) {
            visibleTokens.push({
                type: 'truncation-indicator',
                value: '▶',
                start: endColumn - startColumn,
                end: endColumn - startColumn + 1,
                originalStart: lineText.length,
                originalEnd: lineText.length
            });
        }

        return visibleTokens;
    }
    
}