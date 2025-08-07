import { useEffect, useState } from "react";
import { useStoreSubscribeToField } from "../../../hooks/use-store-subscribe.tsx";
import { EditorStore } from "../stores/editor.store.ts";
import { assemblyOutputStore } from "../../../stores/assembly-output.store.ts";
import { LINE_PADDING_LEFT, LINE_PADDING_TOP, CHAR_HEIGHT } from "../constants.ts";

interface AssemblyErrorDecorationsProps {
    store: EditorStore;
    charWidth: number;
}

interface ParsedError {
    line: number;
    column: number;
    length: number;
    message: string;
}

function parseErrors(errorString: string): ParsedError[] {
    const errors: ParsedError[] = [];
    const lines = errorString.split('\n');
    
    for (const line of lines) {
        // Parse error format: "Line X: message" or "Line X, Column Y: message"
        const lineMatch = line.match(/Line (\d+)(?:, Column (\d+))?: (.+)/);
        if (lineMatch) {
            const lineNum = parseInt(lineMatch[1]) - 1; // Convert to 0-based
            const column = lineMatch[2] ? parseInt(lineMatch[2]) - 1 : 0;
            const message = lineMatch[3];
            
            // Try to extract the problematic token from the error message
            let length = 10; // Default length
            
            // Look for quoted text in the error message
            const quotedMatch = message.match(/['"]([^'"]+)['"]/);
            if (quotedMatch) {
                length = quotedMatch[1].length;
            } else if (message.includes('Invalid operand:') || 
                       message.includes('Unknown mnemonic:') ||
                       message.includes('Invalid register:')) {
                // Extract the problematic token after the colon
                const tokenMatch = message.match(/:\s*(\S+)/);
                if (tokenMatch) {
                    length = tokenMatch[1].length;
                }
            }
            
            errors.push({
                line: lineNum,
                column: column,
                length: length,
                message: message
            });
        }
    }
    
    return errors;
}

export function AssemblyErrorDecorations({ store, charWidth }: AssemblyErrorDecorationsProps) {
    const [errors, setErrors] = useState<ParsedError[]>([]);
    const lines = useStoreSubscribeToField(store.editorState, "lines");
    
    // Subscribe to assembly output errors
    useEffect(() => {
        const subscription = assemblyOutputStore.state.subscribe(state => {
            if (state.error) {
                setErrors(parseErrors(state.error));
            } else {
                setErrors([]);
            }
        });
        
        return () => subscription.unsubscribe();
    }, []);

    if (errors.length === 0) {
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
            {errors.map((error, index) => {
                if (error.line >= lines.length) {
                    return null;
                }
                
                const lineText = lines[error.line].text;
                const startX = error.column * charWidth;
                const width = Math.min(error.length, lineText.length - error.column) * charWidth;
                const y = error.line * CHAR_HEIGHT;
                
                return (
                    <div key={index} className="relative">
                        {/* Invisible hover area that matches the error location */}
                        <div
                            className="absolute group"
                            style={{
                                left: `${startX}px`,
                                top: `${y}px`,
                                width: `${width}px`,
                                height: `${CHAR_HEIGHT}px`,
                                pointerEvents: 'auto'
                            }}
                        >
                            {/* Wavy underline */}
                            <svg
                                className="absolute pointer-events-none"
                                style={{
                                    left: 0,
                                    top: `${CHAR_HEIGHT - 4}px`,
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
                                className="absolute opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-50 pointer-events-none"
                                style={{
                                    left: 0,
                                    top: `${CHAR_HEIGHT + 4}px`,
                                    minWidth: '200px'
                                }}
                            >
                                <div className="bg-zinc-900 border border-red-500 rounded p-2 text-xs text-red-400">
                                    <div className="font-bold mb-1">Assembly Error</div>
                                    <div>{error.message}</div>
                                </div>
                            </div>
                        </div>
                    </div>
                );
            })}
        </div>
    );
}