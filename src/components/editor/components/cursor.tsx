import {useMemo, useRef, useLayoutEffect} from "react";
import clsx from "clsx";
import {useStoreSubscribe, useStoreSubscribeToField} from "../../../hooks/use-store-subscribe.tsx";
import {EditorStore} from "../stores/editor.store.ts";
import {LINE_PADDING_LEFT, LINE_PADDING_TOP, CHAR_HEIGHT} from "../constants.ts";
import {measureCharacterWidth} from "../../helpers.ts";

interface CursorProps {
    store: EditorStore;
}

export function Cursor({store}: CursorProps) {
    const selection = useStoreSubscribeToField(store.editorState, "selection")
    const mode = useStoreSubscribeToField(store.editorState, "mode");
    const focused = useStoreSubscribe(store.focused);
    const isBlinking = useStoreSubscribe(store.cursorBlinkState);
    const isNavigating = useStoreSubscribe(store.isNavigating);

    const cursorRef = useRef<HTMLDivElement>(null);

    const cursorWidth = mode === "insert" ? 2 : 8;
    const cw = useMemo(() => measureCharacterWidth(), []);

    useLayoutEffect(() => {
        if (cursorRef.current) {
            // Use center scrolling when navigating, nearest otherwise
            const scrollBehavior = isNavigating ? "center" : "nearest";
            cursorRef.current.scrollIntoView({block: scrollBehavior, inline: "nearest"});
            
            // Reset navigation flag after scrolling
            if (isNavigating) {
                setTimeout(() => store.isNavigating.next(false), 100);
            }
        }
    }, [selection, isNavigating, store]);

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