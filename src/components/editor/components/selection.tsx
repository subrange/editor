import {useMemo} from "react";
import {useStoreSubscribeToField} from "../../../hooks/use-store-subscribe.tsx";
import {EditorStore} from "../stores/editor.store.ts";
import {LINE_PADDING_LEFT, LINE_PADDING_TOP, CHAR_HEIGHT} from "../constants.ts";

interface SelectionProps {
    store: EditorStore;
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

export function Selection({store}: SelectionProps) {
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