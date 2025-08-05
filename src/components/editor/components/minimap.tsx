import React, {useRef, useEffect, useCallback} from 'react';
import {useStoreSubscribe} from '../../../hooks/use-store-subscribe';
import {type EditorStore} from '../stores/editor.store';
import {type SearchMatch} from '../stores/search.store';
import {type getDimensionsStore} from '../stores/dimensions.store';
import {CHAR_HEIGHT} from '../constants';

type DimensionsStore = ReturnType<typeof getDimensionsStore>;

interface MinimapProps {
    store: EditorStore;
    dimensionsStore: DimensionsStore;
    width?: number;
    scrollTop?: number;
}

export const Minimap = React.memo(function Minimap({store, dimensionsStore, width = 120, scrollTop = 0}: MinimapProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const editorState = useStoreSubscribe(store.editorState);
    const searchMatches = useStoreSubscribe(store.searchStore.state).matches;
    const dimensions = useStoreSubscribe(dimensionsStore.state);
    const lines = editorState.lines;
    const selection = editorState.selection;
    const editorContainerHeight = dimensions.height;

    // Fixed minimap dimensions
    const charWidth = 2;
    const lineHeight = 3;
    const minimapHeight = editorContainerHeight || 500;

    // Calculate scaling
    const totalContentHeight = lines.length * lineHeight;
    const scale = totalContentHeight > minimapHeight ? minimapHeight / totalContentHeight : 1;
    
    // When content is taller than minimap, we need to scroll the minimap content
    const needsScrolling = totalContentHeight > minimapHeight;
    
    // Calculate the offset for scrolling minimap content
    const maxOffset = totalContentHeight - minimapHeight;
    const editorTotalHeight = lines.length * CHAR_HEIGHT;
    const offset = needsScrolling ? (scrollTop / (editorTotalHeight - editorContainerHeight)) * maxOffset : 0;
    
    // Calculate visible area indicator position
    // When minimap scrolls, the indicator position is relative to visible content
    const visibleAreaTop = needsScrolling 
        ? (scrollTop / editorTotalHeight) * totalContentHeight - offset
        : scrollTop * scale;
    const visibleAreaHeight = needsScrolling 
        ? (editorContainerHeight / editorTotalHeight) * totalContentHeight
        : editorContainerHeight * scale;

    // Render minimap content
    useEffect(() => {
        const canvas = canvasRef.current;
        const ctx = canvas?.getContext('2d');
        if (!canvas || !ctx || minimapHeight === 0) return;

        // Set canvas size
        canvas.width = width;
        canvas.height = minimapHeight;

        // Set rendering offset
        ctx.setTransform(1, 0, 0, 1, 0, -offset);

        // Clear canvas
        ctx.fillStyle = '#18181b';
        ctx.fillRect(0, 0, width, minimapHeight);

        // Render lines with scaling
        lines.forEach((line, lineIndex) => {
            const y = lineIndex * lineHeight;
            const text = line.text;

            if (text.trim().length > 0) {
                if (text.trim().startsWith('//')) {
                    if (text.trim().startsWith('// MARK:')) {
                        ctx.fillStyle = '#f59e0b'; // Highlight for MARK comments
                    } else {
                        ctx.fillStyle = '#52525b';
                    } // Color for comments
                } else if (text.trim().startsWith('#define')) {
                    ctx.fillStyle = '#10b981'; // Highlight for #define directives

                } else {
                    ctx.fillStyle = '#6b7280'; // Default text color
                }

                const lineWidth = Math.min(text.length * charWidth, width);
                ctx.fillRect(0, y, lineWidth, lineHeight * 0.8);
            }
        });

        // Render search matches
        if (searchMatches.length > 0) {
            ctx.fillStyle = '#fbbf24';
            searchMatches.forEach((match: SearchMatch) => {
                const y = match.line * lineHeight;
                ctx.fillRect(match.startColumn * charWidth, y, (match.endColumn - match.startColumn) * charWidth, lineHeight);
            });
        }

        // Render cursor line
        const cursorY = selection.focus.line * lineHeight;
        // ctx.fillStyle = 'rgba(168, 85, 247, 1)';
        ctx.fillStyle = 'white';
        ctx.fillRect(0, cursorY, width, lineHeight);

    }, [lines, selection, searchMatches, width, minimapHeight, lineHeight, offset]);

    // Handle click on minimap
    const handleClick = useCallback((e: React.MouseEvent) => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const rect = canvas.getBoundingClientRect();
        const y = e.clientY - rect.top;
        // Account for offset when calculating the line
        const adjustedY = y + offset;
        const line = Math.floor(adjustedY / lineHeight);

        if (line >= 0 && line < lines.length) {
            store.setCursorPosition({line, column: 0});
            store.scrollToCursor();
        }
    }, [lines.length, lineHeight, store, offset]);



    return (
        <div
            className="absolute right-0 top-0 bottom-0 bg-zinc-950 border-l border-zinc-800 opacity-30 hover:opacity-100 transition-opacity"
            style={{width: `${width}px`}}
        >
            <canvas
                ref={canvasRef}
                className="cursor-pointer block"
                onClick={handleClick}
                style={{imageRendering: 'pixelated'}}
            />

            {/* Visible area indicator */}
            <div
                className="absolute left-0 w-full bg-zinc-600/30 border-y border-zinc-500 cursor-ns-resize hover:bg-zinc-500/40 transition-colors"
                style={{
                    top: `${visibleAreaTop}px`,
                    height: `${visibleAreaHeight}px`,
                }}
                onMouseDown={(e) => {
                    e.stopPropagation();
                    e.preventDefault();
                    const startY = e.clientY;
                    const startScrollTop = scrollTop;
                    const currentEditorTotalHeight = lines.length * CHAR_HEIGHT;

                    const handleMouseMove = (e: MouseEvent) => {
                        const deltaY = e.clientY - startY;
                        // Convert pixel movement to editor scroll units
                        const scrollRatio = currentEditorTotalHeight / totalContentHeight;
                        const newScrollTop = startScrollTop + (deltaY * scrollRatio);

                        // Find and scroll the editor element
                        const editorElement = document.querySelector('[data-editor-scroll]') as HTMLElement;
                        if (editorElement) {
                            editorElement.scrollTop = Math.max(0, Math.min(newScrollTop, currentEditorTotalHeight - editorContainerHeight));
                        }
                    };

                    const handleMouseUp = () => {
                        document.removeEventListener('mousemove', handleMouseMove);
                        document.removeEventListener('mouseup', handleMouseUp);
                    };

                    document.addEventListener('mousemove', handleMouseMove);
                    document.addEventListener('mouseup', handleMouseUp);
                }}
            />
        </div>
    );
});