import { useRef, useEffect, useMemo, useCallback, useState } from 'react';
import { useStoreSubscribe } from '../../../hooks/use-store-subscribe';
import { type EditorStore } from '../stores/editor.store';
import { type SearchMatch } from '../stores/search.store';
import clsx from 'clsx';

interface MinimapProps {
    store: EditorStore;
    width?: number;
    scale?: number;
}

export function Minimap({ store, width = 120, scale = 1 }: MinimapProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [containerHeight, setContainerHeight] = useState(0);
    const editorState = useStoreSubscribe(store.editorState);
    const searchMatches = useStoreSubscribe(store.searchStore.state).matches;
    const lines = editorState.lines;
    const selection = editorState.selection;
    
    // Calculate minimap dimensions
    const charWidth = 2 * scale;
    const lineHeight = 3;
    const canvasHeight = Math.max(containerHeight || 300, lines.length * lineHeight);
    
    // Calculate visible area based on editor viewport
    const visibleAreaHeight = 300; // This should match the editor's visible height
    const visibleAreaTop = 0; // This would need to be updated based on editor scroll
    
    // Track container height
    useEffect(() => {
        if (!containerRef.current) return;
        
        const observer = new ResizeObserver((entries) => {
            // Use requestAnimationFrame to avoid infinite loops
            requestAnimationFrame(() => {
                const entry = entries[0];
                if (entry && entry.contentRect.height > 0) {
                    setContainerHeight(entry.contentRect.height);
                }
            });
        });
        
        observer.observe(containerRef.current);
        
        return () => observer.disconnect();
    }, []);
    
    // Render minimap content
    useEffect(() => {
        const canvas = canvasRef.current;
        const ctx = canvas?.getContext('2d');
        if (!canvas || !ctx) return;
        
        // Don't render if container height is 0
        if (containerHeight === 0 && canvasHeight === 0) return;
        
        // Set canvas size
        canvas.width = width;
        canvas.height = canvasHeight;
        
        // Clear canvas with darker background
        ctx.fillStyle = '#18181b'; // zinc-900
        ctx.fillRect(0, 0, width, canvasHeight);
        
        // Render lines
        lines.forEach((line, lineIndex) => {
            const y = lineIndex * lineHeight;
            const text = line.text;
            
            // Render non-empty lines
            if (text.trim().length > 0) {
                // Check if it's a comment line
                if (text.trim().startsWith('//')) {
                    if (text.trim().startsWith('// MARK:')) {
                        ctx.fillStyle = '#f59e0b'; // amber-500 for MARK comments
                    } else {
                        ctx.fillStyle = '#52525b';
                    } // gray-600 for comments
                    ctx.fillRect(0, y, Math.min(text.length * charWidth, width), lineHeight * 0.7);
                } else {
                    // Render individual characters
                    for (let i = 0; i < Math.min(text.length, Math.floor(width / charWidth)); i++) {
                        const char = text[i];
                        if (char !== ' ') {
                            // Color based on character type
                            if ('+-<>[].,'.includes(char)) {
                                ctx.fillStyle = '#3b82f6'; // blue-500 for BF operators
                            } else if (char === '@') {
                                ctx.fillStyle = '#a855f7'; // purple-500 for macros
                            } else if (char === '#') {
                                ctx.fillStyle = '#10b981'; // emerald-500 for preprocessor
                            } else {
                                ctx.fillStyle = '#6b7280'; // gray-500 for other text
                            }
                            
                            ctx.fillRect(i * charWidth, y, charWidth * 0.7, lineHeight * 0.7);
                        }
                    }
                }
            }
        });
        
        // Render search matches
        if (searchMatches.length > 0) {
            ctx.fillStyle = '#fbbf24'; // amber-400
            searchMatches.forEach((match: SearchMatch) => {
                const y = match.line * lineHeight;
                const x = match.startColumn * charWidth;
                const width = (match.endColumn - match.startColumn) * charWidth;
                ctx.fillRect(x, y, width, lineHeight);
            });
        }
        
        // Render cursor line
        const cursorY = selection.focus.line * lineHeight;
        ctx.fillStyle = 'rgba(168, 85, 247, 0.5)'; // purple-500 with opacity
        ctx.fillRect(0, cursorY, width, lineHeight);
        
        // Add a brighter line indicator for the cursor
        ctx.fillStyle = '#a855f7'; // purple-500
        ctx.fillRect(0, cursorY, 2, lineHeight);
        
        // Render selection if exists
        if (selection.anchor.line !== selection.focus.line || selection.anchor.column !== selection.focus.column) {
            ctx.fillStyle = 'rgba(99, 102, 241, 0.3)'; // indigo-500 with opacity
            const startLine = Math.min(selection.anchor.line, selection.focus.line);
            const endLine = Math.max(selection.anchor.line, selection.focus.line);
            
            for (let line = startLine; line <= endLine; line++) {
                const y = line * lineHeight;
                let startX = 0;
                let endX = width;
                
                if (line === selection.anchor.line && line === selection.focus.line) {
                    // Single line selection
                    startX = Math.min(selection.anchor.column, selection.focus.column) * charWidth;
                    endX = Math.max(selection.anchor.column, selection.focus.column) * charWidth;
                } else if (line === startLine) {
                    startX = (selection.anchor.line < selection.focus.line ? selection.anchor.column : selection.focus.column) * charWidth;
                } else if (line === endLine) {
                    endX = (selection.anchor.line < selection.focus.line ? selection.focus.column : selection.anchor.column) * charWidth;
                }
                
                ctx.fillRect(startX, y, endX - startX, lineHeight);
            }
        }
        
    }, [lines, selection, searchMatches, width, canvasHeight, charWidth, lineHeight, containerHeight]);
    
    // Auto-scroll minimap to keep cursor in view
    useEffect(() => {
        if (!containerRef.current) return;
        
        const cursorY = selection.focus.line * lineHeight;
        const container = containerRef.current;
        const scrollTop = container.scrollTop;
        const scrollBottom = scrollTop + container.clientHeight;
        
        // If cursor is outside visible area, scroll to center it
        if (cursorY < scrollTop || cursorY + lineHeight > scrollBottom) {
            container.scrollTop = cursorY - container.clientHeight / 2;
        }
    }, [selection.focus.line, lineHeight]);
    
    // Handle click on minimap
    const handleClick = useCallback((e: React.MouseEvent) => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        
        const rect = canvas.getBoundingClientRect();
        const y = e.clientY - rect.top;
        const line = Math.floor(y / lineHeight);
        
        if (line >= 0 && line < lines.length) {
            store.setCursorPosition({ line, column: 0 });
            store.scrollToCursor();
        }
    }, [lines.length, lineHeight, store]);
    
    // Render visible area indicator
    const visibleAreaStyle = {
        top: `${visibleAreaTop}px`,
        height: `${Math.min(visibleAreaHeight, canvasHeight - visibleAreaTop)}px`,
    };
    
    return (
        <div 
            ref={containerRef}
            className="sticky right-0 top-0 bg-zinc-950 border-l border-zinc-800 flex-shrink-0 overflow-y-auto overflow-x-hidden opacity-20 hover:opacity-100 transition-opacity"
            style={{ width: `${width}px`, height: '100%' }}
        >
            <div className="relative">
                <canvas
                    ref={canvasRef}
                    className="cursor-pointer block"
                    onClick={handleClick}
                    style={{ imageRendering: 'pixelated' }}
                />
                
                {/* Visible area indicator */}
                <div
                    className="absolute left-0 w-full bg-zinc-700/20 border-y border-zinc-600 pointer-events-none"
                    style={visibleAreaStyle}
                />
            </div>
        </div>
    );
}