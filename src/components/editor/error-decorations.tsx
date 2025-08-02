import { useMemo } from "react";
import { useStoreSubscribeToField } from "../../hooks/use-store-subscribe.tsx";
import { EditorStore } from "./editor.store.ts";
import { type MacroExpansionError } from "../../services/macro-expander/macro-expander.ts";
import { CHAR_HEIGHT, LINE_PADDING_LEFT, LINE_PADDING_TOP } from "./constants.ts";

interface ErrorDecorationsProps {
    store: EditorStore;
    errors: MacroExpansionError[];
}

function measureCharacterWidth() {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
        throw new Error("Failed to get canvas context");
    }
    context.font = "14px monospace";
    return context.measureText("M").width;
}

export function ErrorDecorations({ store, errors }: ErrorDecorationsProps) {
    const lines = useStoreSubscribeToField(store.editorState, "lines");
    const cw = useMemo(() => measureCharacterWidth(), []);
    
    // Filter errors that have location information
    const locatedErrors = errors.filter(e => e.location);
    
    if (locatedErrors.length === 0) {
        return null;
    }
    
    return (
        <div 
            className="absolute inset-0 pointer-events-none"
            style={{ 
                paddingLeft: `${LINE_PADDING_LEFT}px`,
                paddingTop: `${LINE_PADDING_TOP}px`
            }}
        >
            {locatedErrors.map((error, index) => {
                if (!error.location || error.location.line >= lines.length) {
                    return null;
                }
                
                const lineText = lines[error.location.line].text;
                const startX = error.location.column * cw;
                const width = Math.min(error.location.length, lineText.length - error.location.column) * cw;
                const y = error.location.line * CHAR_HEIGHT;
                
                return (
                    <div key={index} className="relative group">
                        {/* Wavy underline */}
                        <svg
                            className="absolute"
                            style={{
                                left: `${startX}px`,
                                top: `${y + CHAR_HEIGHT - 4}px`,
                                width: `${width}px`,
                                height: "4px"
                            }}
                        >
                            <path
                                d={`M 0 2 Q 2 0 4 2 T 8 2 T 12 2 T 16 2 T 20 2 T 24 2 T 28 2 T 32 2 T 36 2 T 40 2 T 44 2 T 48 2 T 52 2 T 56 2 T 60 2 T 64 2 T 68 2 T 72 2 T 76 2 T 80 2 T 84 2 T 88 2 T 92 2 T 96 2 T 100 2 T 104 2 T 108 2 T 112 2 T 116 2 T 120 2`}
                                stroke="rgb(239 68 68)"
                                strokeWidth="2"
                                fill="none"
                                className="animate-pulse"
                            />
                        </svg>
                        
                        {/* Error tooltip on hover */}
                        <div
                            className="absolute opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-50 pointer-events-auto"
                            style={{
                                left: `${startX}px`,
                                top: `${y + CHAR_HEIGHT + 4}px`,
                                minWidth: '200px'
                            }}
                        >
                            <div className="bg-zinc-900 border border-red-500 rounded p-2 text-xs text-red-400">
                                <div className="font-bold mb-1 capitalize">
                                    {error.type.replace(/_/g, ' ')}
                                </div>
                                <div>{error.message}</div>
                            </div>
                        </div>
                    </div>
                );
            })}
        </div>
    );
}