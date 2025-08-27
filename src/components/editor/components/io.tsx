import { useRef, useLayoutEffect, useMemo } from "react";

interface IOProps {
    output: string | undefined;
    outputRef?: React.RefObject<HTMLDivElement | null>;
    isActive?: boolean;
    maxLines?: number | null;
}

export function IO({ output, outputRef, isActive = true, maxLines }: IOProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const activeRef = outputRef || containerRef;

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

    return (
        <pre className="text-xs text-white whitespace-pre font-mono">
            {processedOutput}
        </pre>
    );
}