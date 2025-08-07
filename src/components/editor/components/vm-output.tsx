import { useRef, useLayoutEffect } from "react";

interface VMOutputProps {
    outputRef?: React.RefObject<HTMLDivElement | null>;
}

export function VMOutput({ outputRef }: VMOutputProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const activeRef = outputRef || containerRef;

    // Auto-scroll to bottom when content changes
    useLayoutEffect(() => {
        setTimeout(() => {
            if (activeRef.current) {
                activeRef.current.scrollTop = activeRef.current.scrollHeight;
            }
        }, 10);
    }, []);

    return (
        <div ref={activeRef} className="text-xs text-zinc-500 p-4">
            VM Output - Coming soon
        </div>
    );
}