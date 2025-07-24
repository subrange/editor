import {VSep} from "../helper-components.tsx";
import {useStoreSubscribe, useStoreSubscribeToField} from "../../hooks/use-store-subscribe.tsx";
import {editorStore, type Line} from "./editor.store.ts";
import clsx from "clsx";
import {type AppCommand, keybindingsService, type KeybindingState} from "../../services/keybindings.service.ts";
import {useMemo, useRef, useEffect, useLayoutEffect} from "react";
import {Tokenizer, tokenStyles} from "./tokenizer.ts";
import {CHAR_HEIGHT, LINE_PADDING_LEFT, LINE_PADDING_TOP} from "./constants.ts";
import {BracketHighlights} from "./bracket-matcher.tsx";
import {interpreterStore} from "../debugger/interpreter.store.ts";

// Constants for layout measurements


function measureCharacterWidth() {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
        throw new Error("Failed to get canvas context");
    }
    context.font = "14px monospace"; // Match your font-mono text-sm
    const width = context.measureText("M").width;
    console.log(`Character width: ${width}px`);
    return width;
}

function LineNumbersPanel() {
    const editorState = useStoreSubscribe(editorStore.editorState);
    const linesCount = editorState.lines.length;

    return <div
        className="flex flex-col overflow-visible bg-zinc-950 sticky left-0 w-16 min-w-16 text-zinc-700 select-none z-10 py-1">
        {
            Array.from({length: linesCount}, (_, i) => (
                <div key={i} className="text-left pl-2" style={{ height: `${CHAR_HEIGHT}px` }}>
                    {i + 1}
                </div>
            ))
        }
    </div>;
}

function Selection() {
    const selection = useStoreSubscribeToField(editorStore.editorState, "selection");
    const lines = useStoreSubscribeToField(editorStore.editorState, "lines");
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

function Cursor() {
    const selection = useStoreSubscribeToField(editorStore.editorState, "selection")
    const mode = useStoreSubscribeToField(editorStore.editorState, "mode");
    const isBlinking = useStoreSubscribe(editorStore.cursorBlinkState);

    const cursorRef = useRef<HTMLDivElement>(null);

    const cursorWidth = mode === "insert" ? 1 : 8;
    const cw = useMemo(() => measureCharacterWidth(), []);

    useLayoutEffect(() => {
        if (cursorRef.current) {
            cursorRef.current.scrollIntoView({ block: "nearest", inline: "nearest" });
        }
    }, [selection]);

    const stl = {
        left: `${LINE_PADDING_LEFT + selection.focus.column * cw}px`,
        top: `${LINE_PADDING_TOP + selection.focus.line * CHAR_HEIGHT - 3}px`,
        width: `${cursorWidth}px`,
        height: `${CHAR_HEIGHT}px`,
    }

    return <div
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

    useLayoutEffect(() => {
        if (debugMarkerRef.current) {
            debugMarkerRef.current.scrollIntoView({ block: "center" });
        }
    });

    const stl = {
        left: `${LINE_PADDING_LEFT + debugMarkerState.column * cw}px`,
        top: `${LINE_PADDING_TOP + debugMarkerState.line * CHAR_HEIGHT - 3}px`,
        width: `${8}px`,
        height: `${CHAR_HEIGHT}px`,
    }

    return (isRunning || debugMarkerState.line !== 0 || debugMarkerState.column !== 0) && <div
        className={clsx("absolute border border-green-500 pointer-events-none z-10", {

        })}
        style={stl}
        ref={debugMarkerRef}
    />;
}

function LinesPanel() {
    const editorState = useStoreSubscribe(editorStore.editorState);
    const lines = editorState.lines;
    const selection = editorState.selection;

    const containerRef = useRef<HTMLDivElement>(null);
    const charWidth = useMemo(() => measureCharacterWidth(), []);
    const isDraggingRef = useRef(false);
    const dragStartedRef = useRef(false);

    // Create tokenizer instance
    const tokenizerRef = useRef(new Tokenizer());

    // Tokenize all lines whenever content changes
    const tokenizedLines = useMemo(() => {
        const lineTexts = lines.map(l => l.text);
        return tokenizerRef.current.tokenizeAllLines(lineTexts);
    }, [lines]);

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

        return { line, column };
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
                editorStore.updateSelection(position);
            } else {
                editorStore.setCursorPosition(position);
            }
        }
    };

    const handleDoubleClick = (e: React.MouseEvent) => {
        const position = getPositionFromMouse(e);
        if (!position) return;

        editorStore.selectWord(position);
    };

    const handleTripleClick = (e: React.MouseEvent) => {
        const position = getPositionFromMouse(e);
        if (!position) return;

        editorStore.selectLine(position.line);
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
            editorStore.startSelection(position);
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

            editorStore.updateSelection({ line, column });
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

    const renderLine = (line: Line, lineIndex: number) => {
        const tokens = tokenizedLines[lineIndex] || [];

        return (
            <div
                key={lineIndex}
                className="whitespace-pre pl-2 pr-4"
                style={{ height: `${CHAR_HEIGHT}px`, lineHeight: `${CHAR_HEIGHT}px` }}
            >
                {tokens.length === 0 ? (
                    <span>&nbsp;</span>
                ) : (
                    tokens.map((token, tokenIndex) => (
                        <span
                            key={tokenIndex}
                            className={tokenStyles[token.type]}
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
        className="flex flex-col grow-1 overflow-visible py-1 relative cursor-text"
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
        <Selection />
        <BracketHighlights
            cursorPosition={selection.focus}
            lines={lines}
            charWidth={charWidth}
        />
        <Cursor />
        <DebugMarker />
    </div>;
}

export function Editor() {
    const editorRef = useRef<HTMLDivElement>(null);

    function addEditorKeybindings() {
        keybindingsService.pushKeybindings("editor" as KeybindingState, [
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

            // Copy/Cut/Paste
            keybindingsService.createKeybinding("meta+c", "editor.copy" as AppCommand),
            keybindingsService.createKeybinding("meta+x", "editor.cut" as AppCommand),
            keybindingsService.createKeybinding("meta+v", "editor.paste" as AppCommand),
        ])

        editorStore.focus()
    }

    function removeEditorKeybindings() {
        keybindingsService.removeKeybindings("editor" as KeybindingState);
        editorStore.blur();
    }

    // Auto-focus on mount
    // useEffect(() => {
    //     editorRef.current?.focus();
    // }, []);

    return (
        <div
            ref={editorRef}
            className={clsx(
                "flex grow-1 bg-zinc-950 font-mono text-sm inset-shadow-sm overflow-auto relative select-none outline-0",
            )}
            onFocus={addEditorKeybindings}
            tabIndex={0}
            onBlur={removeEditorKeybindings}
        >
            <LineNumbersPanel/>
            <VSep className="sticky left-16 z-1 top-0 bottom-0"></VSep>
            <LinesPanel/>
        </div>
    )
}