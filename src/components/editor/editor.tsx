import {VSep} from "../helper-components.tsx";
import {useStoreSubscribe, useStoreSubscribeToField} from "../../hooks/use-store-subscribe.tsx";
import {editorStore} from "./editor.store.ts";
import clsx from "clsx";
import {type AppCommand, keybindingsService, type KeybindingState} from "../../services/keybindings.service.ts";
import {useMemo} from "react";

function LineNumbersPanel() {
    const editorState = useStoreSubscribe(editorStore.editorState);
    const linesCount = editorState.lines.length;

    return <div /*Line numbers */
        className="v bg-zinc-950 sticky top-0 left-0 w-16 min-w-16 text-zinc-700 select-none z-1 py-1">
        {
            Array.from({length: linesCount}, (_, i) => (
                <div key={i} className="text-left pl-2 my-[1px]">
                    {i + 1}
                </div>
            ))
        }
    </div>;
}

function measureCharacterWidth() {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
        throw new Error("Failed to get canvas context");
    }
    context.font = "14px monospace"; // Adjust font size and family as needed
    const width = context.measureText("M").width; // Measure the width of a character
    console.log(`Character width: ${width}px`);
    return width;
}

function Cursor() {
    const selection = useStoreSubscribeToField(editorStore.editorState, "selection")
    const mode = useStoreSubscribeToField(editorStore.editorState, "mode");

    const isBlinking = useStoreSubscribe(editorStore.cursorBlinkState);

    const cursorWidth = mode === "insert" ? 1 : 8;

    const cw = useMemo(() => measureCharacterWidth(), []);

    const stl = {
        left: `calc(var(--spacing) * 2 + ${selection.focus.column * cw}px)`,
        top: `calc(var(--spacing) * 2 + ${selection.focus.line * 20}px - 1px)`,
        width: `${cursorWidth}px`,
        height: "20px",
    }

    return <div
        className={clsx("absolute bg-zinc-300 mix-blend-difference", {
            "animate-blink": isBlinking,
        })}
        style={stl}
    />;
}

function LinesPanel() {
    const editorState = useStoreSubscribe(editorStore.editorState);
    const lines = editorState.lines;

    return <div /*Editor content */ className="flex flex-col grow-1 overflow-visible py-1 relative cursor-text">
        <div>
            {lines.map((line, index) => (
                <div key={index} className="whitespace-nowrap pl-2 pr-4 my-[1px] text-zinc-300">
                    {line.text}
                </div>
            ))}
        </div>
        <Cursor />
    </div>;
}

export function Editor() {
    function addEditorKeybindings() {
        keybindingsService.pushKeybindings("editor" as KeybindingState, [
            keybindingsService.createKeybinding("arrowright", "editor.moveright" as AppCommand),
            keybindingsService.createKeybinding("arrowleft", "editor.moveleft" as AppCommand),
            keybindingsService.createKeybinding("arrowup", "editor.moveup" as AppCommand),
            keybindingsService.createKeybinding("arrowdown", "editor.movedown" as AppCommand),

            // keybindingsService.createKeybinding("meta+arrowright", "editor.moveright" as AppCommand),
        ])
    }

    function removeEditorKeybindings() {
        keybindingsService.removeKeybindings("editor" as KeybindingState);
    }

    return (
        <div
            className={clsx(
                "flex grow-1 bg-zinc-950 font-mono text-sm inset-shadow-sm overflow-auto relative select-none outline-0",
            )}
            onFocus={addEditorKeybindings}
            tabIndex={0}
            onBlur={removeEditorKeybindings}
        >
            <LineNumbersPanel/>
            <VSep className="sticky left-16 z-1"></VSep>
            <LinesPanel/>
        </div>
    )
}
