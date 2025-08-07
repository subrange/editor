import {useStoreSubscribeToField, useStoreSubscribe} from "../../../hooks/use-store-subscribe.tsx";
import {interpreterStore} from "../../debugger/interpreter-facade.store.ts";
import {useLayoutEffect, useRef, useMemo} from "react";
import clsx from "clsx";
import {ChevronDownIcon, ChevronUpIcon, XMarkIcon} from "@heroicons/react/16/solid";
import {CommandLineIcon} from "@heroicons/react/24/outline";
import {outputStore} from "../../../stores/output.store.ts";

interface OutputProps {
    position?: 'bottom' | 'right' | 'floating';
    showHeader?: boolean;
    onClose?: () => void;
}

export function Output({ position = 'bottom', showHeader = true, onClose }: OutputProps) {
    const outputState = useStoreSubscribe(outputStore.state);
    const output = useStoreSubscribeToField(interpreterStore.state, "output");
    const outputContainer = useRef<HTMLDivElement>(null);
    
    const { collapsed, height, maxLines } = outputState;

    // Process output to handle max lines
    const processedOutput = useMemo(() => {
        if (!maxLines || !output) return output;
        const lines = output.split('\n');
        if (lines.length <= maxLines) return output;
        
        // Keep the last maxLines lines
        const truncated = lines.slice(-maxLines);
        return `[... ${lines.length - maxLines} lines truncated ...]\n${truncated.join('\n')}`;
    }, [output, maxLines]);

    // Scroll to the bottom when output changes
    useLayoutEffect(() => {
        setTimeout(() => {
            if (outputContainer.current && !collapsed) {
                outputContainer.current.scrollTop = outputContainer.current.scrollHeight;
            }
        }, 10);
    }, [processedOutput, collapsed]);

    const containerClasses = clsx(
        "v bg-zinc-900 transition-all",
        {
            // Bottom position styles
            "h-96 min-h-96": position === 'bottom' && !collapsed,
            "h-8 min-h-8": position === 'bottom' && collapsed,
            
            // Right position styles
            "h-full grow-1": position === 'right' && !collapsed,
            "w-8 min-w-8 h-full": position === 'right' && collapsed,
            
            // Floating position styles
            "absolute bottom-4 right-4 w-96 h-64 shadow-2xl rounded-lg border border-zinc-800": position === 'floating',
        }
    );

    const headerClasses = clsx(
        "bg-zinc-900 text-zinc-500 text-xs font-bold p-2 h-8 min-h-8 gap-2",
        "hover:bg-zinc-800 hover:text-zinc-400 transition-colors",
        {
            "h": !collapsed || position !== 'right',
            "v items-center justify-center": collapsed && position === 'right',
            "border-t border-zinc-800": position === 'bottom',
            "border-l border-zinc-800": position === 'right',
            "rounded-t-lg": position === 'floating',
        }
    );

    const contentClasses = clsx(
        "flex flex-col p-2 bg-zinc-950 grow-1 overflow-auto",
        {
            "rounded-b-lg": position === 'floating',
        }
    );

    return (
        <div className={containerClasses} style={{ 
            height: position === 'bottom' && !collapsed ? height : undefined
        }}>
            {showHeader && (
                <button 
                    className={headerClasses}
                    onClick={() => outputStore.setCollapsed(!collapsed)}
                >
                    {collapsed ? (
                        position === 'right' ? (
                            <CommandLineIcon className="w-4 h-4" />
                        ) : (
                            <ChevronUpIcon />
                        )
                    ) : (
                        position === 'right' ? <ChevronUpIcon className="rotate-90" /> : <ChevronDownIcon />
                    )}
                    {(!collapsed || position !== 'right') && <span>Output</span>}
                    {processedOutput && (!collapsed || position !== 'right') && (
                        <span className="text-zinc-600">
                            ({processedOutput.split('\n').length} lines)
                        </span>
                    )}
                    
                    {/* Additional controls when expanded */}
                    {!collapsed && (
                        <div className="ml-auto h gap-2">
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    if (outputContainer.current) {
                                        outputContainer.current.scrollTop = outputContainer.current.scrollHeight;
                                    }
                                }}
                                className="text-zinc-600 hover:text-zinc-400"
                                title="Scroll to bottom"
                            >
                                ↓
                            </button>
                            
                            {position === 'floating' && (
                                <button
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        onClose?.();
                                    }}
                                    className="text-zinc-600 hover:text-zinc-400"
                                >
                                    <XMarkIcon className="w-4 h-4" />
                                </button>
                            )}
                        </div>
                    )}
                </button>
            )}
            
            {!collapsed && (
                <div className={contentClasses} ref={outputContainer}>
                    <pre className="text-xs text-white overflow-x-auto whitespace-pre-wrap font-mono">
                        {processedOutput}
                    </pre>
                </div>
            )}
        </div>
    );
}