import React, {useRef, useEffect, useCallback, useState} from 'react';
import {useStoreSubscribe} from '../../../hooks/use-store-subscribe';
import {type EditorStore} from '../stores/editor.store';
import {type SearchMatch} from '../stores/search.store';
import {type getDimensionsStore} from '../stores/dimensions.store';

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
    const [offset, setOffset] = useState(0);
    const lines = editorState.lines;
    const selection = editorState.selection;
    const editorContainerHeight = dimensions.height;

    // Fixed minimap dimensions
    const charWidth = 2;
    const lineHeight = 3;
    const minimapHeight = editorContainerHeight || 500;

    // Calculate scaling for non-canvas uses
    const totalContentHeight = lines.length * lineHeight;
    const scale = totalContentHeight > minimapHeight ? minimapHeight / totalContentHeight : 1;
    const scaledLineHeight = lineHeight;

    // Calculate visible area
    const visibleAreaTop = scrollTop * scale;
    const visibleAreaHeight = editorContainerHeight * scale;

    // Render minimap content
    useEffect(() => {
        const canvas = canvasRef.current;
        const ctx = canvas?.getContext('2d');
        if (!canvas || !ctx || minimapHeight === 0) return;

        // Set canvas size
        canvas.width = width;
        canvas.height = minimapHeight;

        // Set rendering offset
        const scaledOffset = offset;
        ctx.setTransform(1, 0, 0, 1, 0, -scaledOffset);

        // Clear canvas
        ctx.fillStyle = '#18181b';
        ctx.fillRect(0, 0, width, minimapHeight);

        // Render lines with scaling
        lines.forEach((line, lineIndex) => {
            const y = lineIndex * lineHeight;
            const text = line.text;

            if (text.trim().length > 0) {
                // Simplified rendering for better performance

                if (text.trim().startsWith('//')) {
                    if (text.trim().startsWith('// MARK:')) {
                        ctx.fillStyle = '#f59e0b'; // Highlight for MARK comments
                    } else {
                        ctx.fillStyle = '#52525b';
                    } // Color for comments
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
                const y = match.line * scaledLineHeight;
                ctx.fillRect(match.startColumn * charWidth, y, (match.endColumn - match.startColumn) * charWidth, scaledLineHeight);
            });
        }

        // Render cursor line
        const cursorY = selection.focus.line * scaledLineHeight;
        ctx.fillStyle = 'rgba(168, 85, 247, 0.5)';
        ctx.fillRect(0, cursorY, width, scaledLineHeight);

    }, [lines, selection, searchMatches, width, minimapHeight, lineHeight, offset]);

    // Handle click on minimap
    const handleClick = useCallback((e: React.MouseEvent) => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const rect = canvas.getBoundingClientRect();
        const y = e.clientY - rect.top;
        const line = Math.floor(y / scaledLineHeight);

        if (line >= 0 && line < lines.length) {
            store.setCursorPosition({line, column: 0});
            store.scrollToCursor();
        }
    }, [lines.length, scaledLineHeight, store]);



    return (
        <div
            className="absolute right-0 top-0 bottom-0 bg-zinc-950 border-l border-zinc-800 opacity-50 hover:opacity-100 transition-opacity"
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

                    const handleMouseMove = (e: MouseEvent) => {
                        const deltaY = e.clientY - startY;
                        const newScrollTop = startScrollTop + (deltaY / scale);

                        // Find and scroll the editor element
                        const editorElement = document.querySelector('[data-editor-scroll]') as HTMLElement;
                        if (editorElement) {
                            editorElement.scrollTop = Math.max(0, newScrollTop);
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