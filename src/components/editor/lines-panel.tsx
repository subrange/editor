import {useMemo, useRef, useState, useEffect} from "react";
import {useStoreSubscribe, useStoreSubscribeToField, useStoreSubscribeObservable} from "../../hooks/use-store-subscribe.tsx";
import {EditorStore, type Line} from "./editor.store.ts";
import {ProgressiveMacroTokenizer, type MacroToken} from "./macro-tokenizer-progressive.ts";
import {ErrorDecorations} from "./error-decorations.tsx";
import {type MacroExpansionError, type MacroDefinition} from "../../services/macro-expander/macro-expander.ts";
import {MacroAutocomplete} from "./macro-autocomplete.tsx";
import {LINE_PADDING_LEFT, LINE_PADDING_TOP, CHAR_HEIGHT} from "./constants.ts";
import {BracketHighlights} from "./bracket-matcher.tsx";
import {VirtualizedLine} from "./virtualized-line.tsx";
import {interpreterStore} from "../debugger/interpreter-facade.store.ts";
import {SearchHighlights} from "./search-highlights.tsx";
import {SearchScroll} from "./search-scroll.tsx";
import {Selection} from "./selection.tsx";
import {Cursor} from "./cursor.tsx";
import {DebugMarker} from "./debug-marker.tsx";

interface LinesPanelProps {
    store: EditorStore;
    editorWidth: number;
    scrollLeft: number;
}

function measureCharacterWidth() {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
        throw new Error("Failed to get canvas context");
    }
    context.font = "14px monospace"; // Match your font-mono text-sm
    const width = context.measureText("M").width;
    return width;
}

export function LinesPanel({store, editorWidth, scrollLeft}: LinesPanelProps) {
    const editorState = useStoreSubscribe(store.editorState);
    const lines = editorState.lines;
    const selection = editorState.selection;

    const containerRef = useRef<HTMLDivElement>(null);
    const charWidth = useMemo(() => measureCharacterWidth(), []);
    const isDraggingRef = useRef(false);
    const dragStartedRef = useRef(false);
    const [isMetaKeyHeld, setIsMetaKeyHeld] = useState(false);
    const [macroExpansionVersion, setMacroExpansionVersion] = useState(0);

    const breakpoints = useStoreSubscribeToField(interpreterStore.state, "breakpoints");
    const expandedLine = useStoreSubscribeToField(interpreterStore.currentChar, "line");
    const sourceLine = useStoreSubscribeObservable(interpreterStore.currentSourceChar, false, null);
    const isRunning = useStoreSubscribeToField(interpreterStore.state, "isRunning");
    
    // Use source position for macro editor when available
    const isMacroEditor = store.getId() === 'macro';
    const currentDebuggingLine = (isMacroEditor && sourceLine) ? sourceLine.line : expandedLine;

    // Get tokenizer from store
    const tokenizer = store.getTokenizer();


    // Subscribe to tokenizer state changes if it's an enhanced macro tokenizer
    useEffect(() => {
        if (tokenizer instanceof ProgressiveMacroTokenizer) {
            const unsubscribe = tokenizer.onStateChange(() => {
                // Force re-render by updating version
                setMacroExpansionVersion(v => v + 1);
            });
            return unsubscribe;
        }
    }, [tokenizer]);

    // Tokenize all lines whenever content changes
    const tokenizedLines = useMemo(() => {
        const lineTexts = lines.map(l => l.text);
        return tokenizer.tokenizeAllLines(lineTexts);
    }, [lines, tokenizer]); // Remove macroExpansionVersion - we don't need to re-tokenize

    // Determine which token styles to use based on tokenizer type
    const isProgressiveMacro = tokenizer instanceof ProgressiveMacroTokenizer;

    // Extract errors and macros if using enhanced tokenizer
    const errors: MacroExpansionError[] = useMemo(() => {
        if (isProgressiveMacro && (tokenizer as ProgressiveMacroTokenizer).state) {
            const errs = (tokenizer as ProgressiveMacroTokenizer).state.expanderErrors || [];
            return errs;
        }
        return [];
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [isProgressiveMacro, tokenizer, macroExpansionVersion]); // macroExpansionVersion forces re-render when tokenizer state changes

    const availableMacros: MacroDefinition[] = useMemo(() => {
        if (isProgressiveMacro && (tokenizer as ProgressiveMacroTokenizer).state) {
            return (tokenizer as ProgressiveMacroTokenizer).state.macroDefinitions || [];
        }
        return [];
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [isProgressiveMacro, tokenizer, macroExpansionVersion]); // macroExpansionVersion forces re-render when tokenizer state changes


    // Track cmd/ctrl key state
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.metaKey || e.ctrlKey) {
                setIsMetaKeyHeld(true);
            }
        };

        const handleKeyUp = (e: KeyboardEvent) => {
            if (!e.metaKey && !e.ctrlKey) {
                setIsMetaKeyHeld(false);
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('keyup', handleKeyUp);

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('keyup', handleKeyUp);
        };
    }, []);

    // Helper to convert mouse position to text position
    const getPositionFromMouse = (e: React.MouseEvent) => {
        if (!containerRef.current) return null;

        const rect = containerRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left - LINE_PADDING_LEFT;
        const y = e.clientY - rect.top - LINE_PADDING_TOP;

        // Calculate line number
        let line = Math.floor(y / CHAR_HEIGHT);
        line = Math.max(0, Math.min(line, lines.length - 1));

        // Calculate column
        let column = Math.round(x / charWidth);
        column = Math.max(0, Math.min(column, lines[line].text.length));

        return {line, column};
    };

    const handleClick = (e: React.MouseEvent) => {
        // Ignore click if we just finished dragging
        if (isDraggingRef.current) {
            isDraggingRef.current = false;
            return;
        }

        // Only handle single clicks (not part of double/triple click)
        if (e.detail === 1) {
            const position = getPositionFromMouse(e);
            if (!position) return;

            // Check if shift is held for extending selection
            if (e.shiftKey) {
                store.updateSelection(position);
            } else {
                store.setCursorPosition(position);
            }
        }
    };

    const handleDoubleClick = (e: React.MouseEvent) => {
        const position = getPositionFromMouse(e);
        if (!position) return;

        store.selectWord(position);
    };

    const handleTripleClick = (e: React.MouseEvent) => {
        const position = getPositionFromMouse(e);
        if (!position) return;

        store.selectLine(position.line);
    };

    const handleMouseDown = (e: React.MouseEvent) => {
        // Only start drag selection on single click
        if (e.detail !== 1) return;
        // And on the left mouse button
        if (e.button !== 0) return;

        const position = getPositionFromMouse(e);
        if (!position) return;

        // Don't start new selection if shift is held
        if (!e.shiftKey) {
            store.startSelection(position);
        }

        dragStartedRef.current = false;

        // Add mouse move and up listeners for selection
        const handleMouseMove = (e: MouseEvent) => {
            const rect = containerRef.current?.getBoundingClientRect();
            if (!rect) return;

            // Mark that we're actually dragging (not just clicking)
            if (!dragStartedRef.current) {
                dragStartedRef.current = true;
                isDraggingRef.current = true;
            }

            const x = e.clientX - rect.left - LINE_PADDING_LEFT;
            const y = e.clientY - rect.top - LINE_PADDING_TOP;

            let line = Math.floor(y / CHAR_HEIGHT);
            line = Math.max(0, Math.min(line, lines.length - 1));

            let column = Math.round(x / charWidth);
            column = Math.max(0, Math.min(column, lines[line].text.length));

            store.updateSelection({line, column});
        };

        const handleMouseUp = () => {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);

            // Only set isDraggingRef if we actually moved the mouse
            if (!dragStartedRef.current) {
                isDraggingRef.current = false;
            }
        };

        document.addEventListener('mousemove', handleMouseMove);
        document.addEventListener('mouseup', handleMouseUp);
    };

    const handleTokenClick = (e: React.MouseEvent, token: MacroToken) => {
        // Check if cmd/ctrl is held and we're clicking on a macro invocation
        if ((e.metaKey || e.ctrlKey) && token.type === 'macro_invocation' && isProgressiveMacro) {
            e.preventDefault();
            e.stopPropagation();

            // Extract macro name from the token value (remove @ and parameters)
            const macroName = token.value.match(/^@([a-zA-Z_]\w*)/)?.[1];
            if (!macroName) {
                return;
            }

            // Find the macro definition
            const macroDef = availableMacros.find(m => m.name === macroName);
            if (macroDef && macroDef.sourceLocation) {
                // Set navigation flag for center scrolling
                store.isNavigating.next(true);
                // Jump to the macro definition
                store.setCursorPosition({
                    line: macroDef.sourceLocation.line,
                    column: macroDef.sourceLocation.column
                });
            }
        }
    };

    const renderLine = (line: Line, lineIndex: number) => {
        const tokens = tokenizedLines[lineIndex] || [];
        let hasBreakpoint = false;
        
        if (isMacroEditor) {
            // For macro editor, use the same logic as line numbers panel
            const trimmed = line.text.trim();
            if (trimmed.length > 0 && !trimmed.startsWith('//')) {
                const firstNonWhitespace = line.text.search(/\S/);
                if (firstNonWhitespace >= 0) {
                    hasBreakpoint = interpreterStore.hasSourceBreakpointAt({line: lineIndex, column: firstNonWhitespace});
                }
            }
        } else {
            // For main editor, check regular breakpoints
            hasBreakpoint = breakpoints.some(bp => bp.line === lineIndex);
        }
        
        const isCurrentLine = currentDebuggingLine === lineIndex;

        return (
            <VirtualizedLine
                key={lineIndex}
                tokens={tokens}
                lineText={line.text}
                lineIndex={lineIndex}
                charWidth={charWidth}
                isProgressiveMacro={isProgressiveMacro}
                hasBreakpoint={hasBreakpoint}
                isCurrentLine={isCurrentLine}
                isRunning={isRunning}
                showDebug={store.showDebug}
                onTokenClick={handleTokenClick}
                isMetaKeyHeld={isMetaKeyHeld}
                editorWidth={editorWidth || 1000}
                editorScrollLeft={scrollLeft}
            />
        );
    };

    return <div
        ref={containerRef}
        className="flex flex-col grow-1 overflow-visible py-1 relative cursor-text min-h-0 pb-24"
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
        onMouseDown={handleMouseDown}
        onMouseUp={(e) => {
            if (e.detail === 3) {
                handleTripleClick(e);
            }
        }}
    >
        <div className="">
            {lines.map(renderLine)}
        </div>
        <Selection store={store}/>
        <BracketHighlights
            cursorPosition={selection.focus}
            lines={lines}
            charWidth={charWidth}
        />
        <SearchHighlights
            searchStore={store.searchStore}
            charWidth={charWidth}
        />
        <SearchScroll
            searchStore={store.searchStore}
            charWidth={charWidth}
        />
        {isProgressiveMacro && errors.length > 0 && (
            <ErrorDecorations store={store} errors={errors}/>
        )}
        {isProgressiveMacro && (
            <MacroAutocomplete
                store={store}
                macros={availableMacros}
                charWidth={charWidth}
            />
        )}
        <Cursor store={store}/>
        {
            store.showDebug && <DebugMarker store={store}/>
        }
    </div>;
}