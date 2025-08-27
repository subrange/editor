import { useRef, useLayoutEffect, useMemo, useEffect, useState } from "react";
import { interpreterStore } from "../../debugger/interpreter-facade.store";
import { outputStore } from "../../../stores/output.store";

interface IOProps {
    output: string | undefined;
    outputRef?: React.RefObject<HTMLDivElement | null>;
    isActive?: boolean;
    maxLines?: number | null;
}

export function IO({ output, outputRef, isActive = true, maxLines }: IOProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const activeRef = outputRef || containerRef;
    const [isWaitingForInput, setIsWaitingForInput] = useState(false);

    // Subscribe to interpreter state for input waiting status
    useEffect(() => {
        const subscription = interpreterStore.state.subscribe(state => {
            setIsWaitingForInput(state.isWaitingForInput || false);
        });
        return () => subscription.unsubscribe();
    }, []);

    // Process output to handle max lines
    const processedOutput = useMemo(() => {
        if (!maxLines || !output) return output;
        const lines = output.split('\n');
        if (lines.length <= maxLines) return output;
        
        // Keep the last maxLines lines
        const truncated = lines.slice(-maxLines);
        return `[... ${lines.length - maxLines} lines truncated ...]\n${truncated.join('\n')}`;
    }, [output, maxLines]);

    // Auto-scroll to bottom when content changes
    useLayoutEffect(() => {
        if (!isActive) return;
        
        setTimeout(() => {
            if (activeRef.current) {
                activeRef.current.scrollTop = activeRef.current.scrollHeight;
            }
        }, 10);
    }, [processedOutput, isActive, activeRef]);

    // Handle keyboard input when waiting
    useEffect(() => {
        if (!isWaitingForInput) return;

        console.log('IO: Input requested, checking if need to show panel');
        
        // Show output panel if collapsed
        const outputState = outputStore.state.getValue();
        if (outputState.collapsed) {
            console.log('IO: Panel is collapsed, showing it');
            outputStore.setCollapsed(false);
        }

        // Only handle input if this tab is active
        if (!isActive) {
            console.log('IO: Tab not active, not handling input');
            return;
        }

        // Focus the container to capture keyboard input
        if (activeRef.current) {
            activeRef.current.focus();
        }

        const handleKeyPress = (e: KeyboardEvent) => {
            if (e.key.length === 1) {
                console.log(`IO: Received input '${e.key}' (ASCII ${e.key.charCodeAt(0)})`);
                
                // Single character input
                // Check if WASM interpreter is running
                const rustWasm = (window as any).rustWasmInterpreter;
                if (rustWasm && rustWasm.isWaitingForInput$ && rustWasm.isWaitingForInput$.getValue()) {
                    console.log('IO: Providing input to WASM interpreter');
                    rustWasm.provideInput(e.key);
                } else {
                    console.log('IO: Providing input to JS/Worker interpreter');
                    (interpreterStore as any).provideInput(e.key);
                }
                e.preventDefault();
            }
        };

        window.addEventListener('keypress', handleKeyPress);
        return () => {
            window.removeEventListener('keypress', handleKeyPress);
        };
    }, [isWaitingForInput, isActive, activeRef]);

    return (
        <div 
            tabIndex={-1} 
            ref={containerRef} 
            className={`outline-none ${isWaitingForInput ? 'ring-0 ring-zinc-500 ring-opacity-50 rounded-sm' : ''}`}
        >
            <pre className="text-xs text-white whitespace-pre font-mono">
                {processedOutput}
                {isWaitingForInput && (
                    <>
                        <span className="animate-pulse text-blue-400">_</span>
                        <span className="text-zinc-500 ml-2">(Type a character for input)</span>
                    </>
                )}
            </pre>
        </div>
    );
}