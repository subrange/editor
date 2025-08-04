import {VSep} from "../helper-components.tsx";
import {useStoreSubscribe, useStoreSubscribeToField} from "../../hooks/use-store-subscribe.tsx";
import {EditorStore, type Line} from "./editor.store.ts";
import clsx from "clsx";
import {type AppCommand, keybindingsService, type KeybindingState} from "../../services/keybindings.service.ts";
import {useMemo, useRef, useLayoutEffect, useState, useEffect} from "react";
import {tokenStyles} from "./tokenizer.ts";
import {progressiveMacroTokenStyles, ProgressiveMacroTokenizer, type MacroToken} from "./macro-tokenizer-progressive.ts";
import {ErrorDecorations} from "./error-decorations.tsx";
import {type MacroExpansionError, type MacroDefinition} from "../../services/macro-expander/macro-expander.ts";
import {MacroAutocomplete} from "./macro-autocomplete.tsx";
import {CHAR_HEIGHT, LINE_PADDING_LEFT, LINE_PADDING_TOP} from "./constants.ts";
import {BracketHighlights} from "./bracket-matcher.tsx";
import {interpreterStore} from "../debugger/interpreter-facade.store.ts";

// Constants for layout measurements

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

interface LineNumbersPanelProps {
    store: EditorStore;
}

function LineNumbersPanel({ store }: LineNumbersPanelProps) {
    const editorState = useStoreSubscribe(store.editorState);
    const currentChar = useStoreSubscribe(interpreterStore.currentChar);
    const breakpoints = useStoreSubscribeToField(interpreterStore.state, "breakpoints");

    const handleLineClick = (lineIndex: number) => {
        if (!store.showDebug) {
            return;
        }

        const line = editorState.lines[lineIndex];
        if (!line) return;

        for (let i = 0; i < line.text.length; i++) {
            if ('><+-[].,$'.includes(line.text[i])) {
                interpreterStore.toggleBreakpoint({line: lineIndex, column: i});
                break;
            }
        }
    };

    return (
        <div
            className="flex flex-col overflow-visible bg-zinc-950 sticky left-0 w-16 min-w-16 min-h-0 text-zinc-700 select-none z-1 py-1">
            {editorState.lines.map((_, i) => {
                const hasBreakpoint = breakpoints.some(bp => bp.line === i);
                const isCurrentLine = currentChar.line === i;

                return (
                    <div
                        key={i}
                        className={`flex justify-between align-center px-2  hover:bg-zinc-800 ${
                            (store.showDebug && isCurrentLine) ? 'bg-zinc-800 text-zinc-300' : ''
                        }`}
                        onClick={() => handleLineClick(i)}
                    >
                        {(store.showDebug && hasBreakpoint) ? <span className="text-red-500 mr-1">●</span> : <span/>}
                        {i + 1}
                    </div>
                );
            })}
        </div>
    );
}

interface SelectionProps {
    store: EditorStore;
}

function Selection({ store }: SelectionProps) {
    const selection = useStoreSubscribeToField(store.editorState, "selection");
    const lines = useStoreSubscribeToField(store.editorState, "lines");
    const cw = useMemo(() => measureCharacterWidth(), []);

    // Don't render if selection is collapsed
    if (selection.anchor.line === selection.focus.line &&
        selection.anchor.column === selection.focus.column) {
        return null;
    }

    // Normalize selection (start is always before end)
    const start = selection.anchor;
    const end = selection.focus;

    let normalizedStart, normalizedEnd;
    if (start.line < end.line || (start.line === end.line && start.column <= end.column)) {
        normalizedStart = start;
        normalizedEnd = end;
    } else {
        normalizedStart = end;
        normalizedEnd = start;
    }

    // Generate selection rectangles
    const rects = [];

    if (normalizedStart.line === normalizedEnd.line) {
        // Single line selection
        rects.push({
            left: LINE_PADDING_LEFT + normalizedStart.column * cw,
            top: LINE_PADDING_TOP + normalizedStart.line * CHAR_HEIGHT - 3,
            width: (normalizedEnd.column - normalizedStart.column) * cw,
            height: CHAR_HEIGHT
        });
    } else {
        // Multi-line selection
        // First line
        const firstLineLength = lines[normalizedStart.line].text.length;
        rects.push({
            left: LINE_PADDING_LEFT + normalizedStart.column * cw,
            top: LINE_PADDING_TOP + normalizedStart.line * CHAR_HEIGHT - 3,
            width: (firstLineLength - normalizedStart.column) * cw,
            height: CHAR_HEIGHT
        });

        // Middle lines
        for (let i = normalizedStart.line + 1; i < normalizedEnd.line; i++) {
            const lineLength = lines[i].text.length;
            rects.push({
                left: LINE_PADDING_LEFT,
                top: LINE_PADDING_TOP + i * CHAR_HEIGHT - 3,
                width: lineLength * cw || cw, // At least one char width for empty lines
                height: CHAR_HEIGHT
            });
        }

        // Last line
        rects.push({
            left: LINE_PADDING_LEFT,
            top: LINE_PADDING_TOP + normalizedEnd.line * CHAR_HEIGHT - 3,
            width: normalizedEnd.column * cw,
            height: CHAR_HEIGHT
        });
    }

    return (
        <>
            {rects.map((rect, index) => (
                <div
                    key={index}
                    className="absolute bg-blue-500 opacity-30 pointer-events-none"
                    style={{
                        left: `${rect.left}px`,
                        top: `${rect.top}px`,
                        width: `${rect.width}px`,
                        height: `${rect.height}px`,
                    }}
                />
            ))}
        </>
    );
}

interface CursorProps {
    store: EditorStore;
}

function Cursor({ store }: CursorProps) {
    const selection = useStoreSubscribeToField(store.editorState, "selection")
    const mode = useStoreSubscribeToField(store.editorState, "mode");
    const focused = useStoreSubscribe(store.focused);
    const isBlinking = useStoreSubscribe(store.cursorBlinkState);

    const cursorRef = useRef<HTMLDivElement>(null);

    const cursorWidth = mode === "insert" ? 2 : 8;
    const cw = useMemo(() => measureCharacterWidth(), []);

    useLayoutEffect(() => {
        if (cursorRef.current) {
            cursorRef.current.scrollIntoView({block: "nearest", inline: "nearest"});
        }
    }, [selection]);

    const stl = {
        left: `${LINE_PADDING_LEFT + selection.focus.column * cw}px`,
        top: `${LINE_PADDING_TOP + selection.focus.line * CHAR_HEIGHT - 3}px`,
        width: `${cursorWidth}px`,
        height: `${CHAR_HEIGHT}px`,
    }

    return focused && <div
        className={clsx("absolute bg-zinc-300 mix-blend-difference pointer-events-none z-10", {
            "animate-blink": isBlinking,
        })}
        style={stl}
        ref={cursorRef}
    />;
}

function DebugMarker() {
    const debugMarkerState = useStoreSubscribe(interpreterStore.currentChar);
    const cw = useMemo(() => measureCharacterWidth(), []);
    const debugMarkerRef = useRef<HTMLDivElement>(null);

    const isRunning = useStoreSubscribeToField(interpreterStore.state, "isRunning");
    const isFinished = useStoreSubscribeToField(interpreterStore.state, "isStopped");

    useLayoutEffect(() => {
        if (debugMarkerRef.current && (isRunning || !isFinished)) {
            debugMarkerRef.current.scrollIntoView({block: "center"});
        }
    });

    const stl = {
        left: `${LINE_PADDING_LEFT + debugMarkerState.column * cw}px`,
        top: `${LINE_PADDING_TOP + debugMarkerState.line * CHAR_HEIGHT - 3}px`,
        width: `${8}px`,
        height: `${CHAR_HEIGHT}px`,
    }

    return (isRunning || debugMarkerState.line !== 0 || debugMarkerState.column !== 0) && <div
        className={clsx("absolute border border-green-500 pointer-events-none z-10", {})}
        style={stl}
        ref={debugMarkerRef}
    />;
}

interface LinesPanelProps {
    store: EditorStore;
}

function LinesPanel({ store }: LinesPanelProps) {
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
    const currentDebuggingLine = useStoreSubscribeToField(interpreterStore.currentChar, "line");
    const isRunning = useStoreSubscribeToField(interpreterStore.state, "isRunning");

    // Get tokenizer from store
    const tokenizer = store.getTokenizer();

    console.log('RERENDER')

    // Subscribe to tokenizer state changes if it's an enhanced macro tokenizer
    useEffect(() => {
        if (tokenizer instanceof ProgressiveMacroTokenizer) {
            console.log('Subscribing to tokenizer state changes');
            const unsubscribe = tokenizer.onStateChange(() => {
                console.log('Tokenizer state changed, forcing re-render');
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
    const styles = isProgressiveMacro ? progressiveMacroTokenStyles : tokenStyles;
    
    // Extract errors and macros if using enhanced tokenizer
    const errors: MacroExpansionError[] = useMemo(() => {
        if (isProgressiveMacro && (tokenizer as ProgressiveMacroTokenizer).state) {
            const errs = (tokenizer as ProgressiveMacroTokenizer).state.expanderErrors || [];
            console.log('Errors in editor:', errs, 'version:', macroExpansionVersion);
            return errs;
        }
        return [];
    }, [isProgressiveMacro, tokenizer, macroExpansionVersion]);
    
    const availableMacros: MacroDefinition[] = useMemo(() => {
        if (isProgressiveMacro && (tokenizer as ProgressiveMacroTokenizer).state) {
            return (tokenizer as ProgressiveMacroTokenizer).state.macroDefinitions || [];
        }
        return [];
    }, [isProgressiveMacro, tokenizer]);

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
                console.log('Could not extract macro name from:', token.value);
                return;
            }
            
            console.log('Looking for macro:', macroName);
            console.log('Available macros:', availableMacros.map(m => m.name));
            
            // Find the macro definition
            const macroDef = availableMacros.find(m => m.name === macroName);
            if (macroDef && macroDef.sourceLocation) {
                console.log('Jumping to:', macroDef.sourceLocation);
                // Jump to the macro definition
                store.setCursorPosition({
                    line: macroDef.sourceLocation.line,
                    column: macroDef.sourceLocation.column
                });
            } else {
                console.log('Macro definition not found or has no source location');
            }
        }
    };

    const renderLine = (_line: Line, lineIndex: number) => {
        const tokens = tokenizedLines[lineIndex] || []

        const strippedTokens = tokens.length > 500
            ? tokens.slice(0, 500).concat({type: 'comment', value: '... (truncated)', start: 0, end: 0})
            : tokens;

        const hasBreakpoint = breakpoints.some(bp => bp.line === lineIndex);
        const isCurrentLine = currentDebuggingLine === lineIndex;

        return (
            <div
                key={lineIndex}
                className={clsx(
                    "whitespace-pre pl-2 pr-4", {
                        "bg-zinc-900": store.showDebug && isCurrentLine && isRunning && !hasBreakpoint,
                        "bg-red-950": store.showDebug && hasBreakpoint
                    }
                )}
                style={{height: `${CHAR_HEIGHT}px`, lineHeight: `${CHAR_HEIGHT}px`}}
            >
                {strippedTokens.length === 0 ? (
                    <span>&nbsp;</span>
                ) : (
                    strippedTokens.map((token, tokenIndex) => (
                        <span
                            key={tokenIndex}
                            className={clsx(styles[token.type as keyof typeof styles] || '', {
                                'cursor-pointer hover:underline': isMetaKeyHeld && token.type === 'macro_invocation' && isProgressiveMacro
                            })}
                            onClick={(e) => handleTokenClick(e, token)}
                        >
                            {token.value}
                        </span>
                    ))
                )}
            </div>
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
        {isProgressiveMacro && errors.length > 0 && (
            <ErrorDecorations store={store} errors={errors} />
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
            store.showDebug && <DebugMarker/>
        }
    </div>;
}

export interface EditorProps {
    store: EditorStore;
    onFocus?: () => void;
    onBlur?: () => void;
}

export function Editor({ store, onFocus, onBlur }: EditorProps) {
    const editorRef = useRef<HTMLDivElement>(null);
    const focused = useStoreSubscribe(store.focused);


    function addEditorKeybindings() {
        // Use editor-specific keybinding state to avoid conflicts
        const keybindingState = `editor_${store.getId()}` as KeybindingState;
        keybindingsService.pushKeybindings(keybindingState, [
            keybindingsService.createKeybinding("arrowright", "editor.moveright" as AppCommand),
            keybindingsService.createKeybinding("arrowleft", "editor.moveleft" as AppCommand),
            keybindingsService.createKeybinding("arrowup", "editor.moveup" as AppCommand),
            keybindingsService.createKeybinding("arrowdown", "editor.movedown" as AppCommand),

            keybindingsService.createKeybinding("meta+z", "editor.undo" as AppCommand),
            keybindingsService.createKeybinding("meta+y", "editor.redo" as AppCommand),
            keybindingsService.createKeybinding("meta+shift+z", "editor.redo" as AppCommand),

            // Add selection keybindings
            keybindingsService.createKeybinding("meta+a", "editor.selectall" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowright", "editor.selectright" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowright", "editor.selectwordright" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowleft", "editor.selectleft" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowleft", "editor.selectwordleft" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowup", "editor.selectup" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowup", "editor.selectlineup" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowdown", "editor.selectdown" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowdown", "editor.selectlinedown" as AppCommand),
            
            // Line start/end movement
            keybindingsService.createKeybinding("meta+arrowleft", "editor.movelinestart" as AppCommand),
            keybindingsService.createKeybinding("meta+arrowright", "editor.movelineend" as AppCommand),
            keybindingsService.createKeybinding("meta+shift+arrowleft", "editor.selectlinestart" as AppCommand),
            keybindingsService.createKeybinding("meta+shift+arrowright", "editor.selectlineend" as AppCommand),

            // Copy/Cut/Paste
            keybindingsService.createKeybinding("meta+c", "editor.copy" as AppCommand),
            keybindingsService.createKeybinding("meta+x", "editor.cut" as AppCommand),
            keybindingsService.createKeybinding("meta+v", "editor.paste" as AppCommand),
        ])

        store.focus();
        onFocus?.();
    }

    function removeEditorKeybindings() {
        const keybindingState = `editor_${store.getId()}` as KeybindingState;
        keybindingsService.removeKeybindings(keybindingState);
        store.blur();
        onBlur?.();
    }

    return (
        <div
            ref={editorRef}
            className={clsx(
                "flex grow-1 bg-zinc-950 font-mono text-sm inset-shadow-sm overflow-auto relative select-none outline-0",
                {
                    "border border-zinc-700": focused,
                    "border border-transparent": !focused
                }
            )}
            onFocus={addEditorKeybindings}
            tabIndex={0}
            onBlur={removeEditorKeybindings}
        >
            <div className="flex relative grow-1 overflow-visible min-h-0 h-fit relative">
                <LineNumbersPanel store={store}/>
                <VSep className="sticky left-16 z-1 top-0 bottom-0"></VSep>
                <LinesPanel store={store}/>
            </div>
        </div>
    )
}