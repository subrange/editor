import {useStoreSubscribeToField, useStoreSubscribe} from "../../../hooks/use-store-subscribe.tsx";
import {interpreterStore} from "../../debugger/interpreter-facade.store.ts";
import {useLayoutEffect, useRef, useMemo} from "react";
import clsx from "clsx";
import {ChevronDownIcon, ChevronUpIcon, XMarkIcon} from "@heroicons/react/16/solid";
import {CommandLineIcon} from "@heroicons/react/24/outline";
import {outputStore} from "../../../stores/output.store.ts";
import {VMOutput} from "./vm-output.tsx";
import {useLocalStorageState} from "../../../hooks/use-local-storage-state.tsx";

interface OutputProps {
    position?: 'bottom' | 'right' | 'floating';
    showHeader?: boolean;
    onClose?: () => void;
}

export function Output({ position = 'bottom', showHeader = true, onClose }: OutputProps) {
    const [activeTab, setActiveTab] = useLocalStorageState<'output' | 'vm'>('output-panel-active-tab', 'output');
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
        "bg-zinc-900 text-zinc-500 text-xs font-bold h-8 min-h-8",
        {
            "h": !collapsed || position !== 'right',
            "v items-center justify-center": collapsed && position === 'right',
            "border-t border-zinc-800": position === 'bottom',
            "border-l border-zinc-800": position === 'right',
            "rounded-t-lg": position === 'floating',
        }
    );
    
    const tabButtonClasses = (isActive: boolean) => clsx(
        "px-3 py-2 text-xs font-bold transition-colors",
        {
            "text-zinc-400 bg-zinc-800": isActive,
            "text-zinc-600 hover:text-zinc-500 hover:bg-zinc-800/50": !isActive,
        }
    );

    const contentClasses = clsx(
        "flex flex-col p-2 bg-zinc-950 grow-1",
        {
            "rounded-b-lg": position === 'floating',
        }
    );

    return (
        <div className={containerClasses} style={{ 
            height: position === 'bottom' && !collapsed ? height : undefined
        }}>
            {showHeader && (
                <div className={headerClasses}>
                    {collapsed ? (
                        // When collapsed, show a simple button
                        <button 
                            className="w-full h-full flex items-center justify-center gap-2 hover:bg-zinc-800 transition-colors"
                            onClick={() => outputStore.setCollapsed(false)}
                        >
                            {position === 'right' ? (
                                <CommandLineIcon className="w-4 h-4" />
                            ) : (
                                <>
                                    <ChevronUpIcon className="w-4 h-4" />
                                    <span>Output</span>
                                </>
                            )}
                        </button>
                    ) : (
                        // When expanded, show full header with tabs
                        <>
                            {/* Collapse button */}
                            <button 
                                className="p-2 hover:bg-zinc-800 transition-colors"
                                onClick={() => outputStore.setCollapsed(true)}
                                title="Collapse panel"
                            >
                                {position === 'right' ? <ChevronUpIcon className="w-4 h-4 rotate-90" /> : <ChevronDownIcon className="w-4 h-4" />}
                            </button>
                            
                            {/* Tabs */}
                            <div className="h gap-0">
                                <button
                                    className={tabButtonClasses(activeTab === 'output')}
                                    onClick={() => setActiveTab('output')}
                                >
                                    Output
                                </button>
                                <button
                                    className={tabButtonClasses(activeTab === 'vm')}
                                    onClick={() => setActiveTab('vm')}
                                >
                                    VM Output
                                </button>
                            </div>
                            
                            {/* Additional controls */}
                            <div className="ml-auto h gap-2 p-2">
                                {activeTab === 'output' && (
                                    <button
                                        onClick={() => {
                                            if (outputContainer.current) {
                                                outputContainer.current.scrollTop = outputContainer.current.scrollHeight;
                                            }
                                        }}
                                        className="text-zinc-600 hover:text-zinc-400"
                                        title="Scroll to bottom"
                                    >
                                        ↓
                                    </button>
                                )}
                                
                                {position === 'floating' && (
                                    <button
                                        onClick={() => onClose?.()}
                                        className="text-zinc-600 hover:text-zinc-400"
                                    >
                                        <XMarkIcon className="w-4 h-4" />
                                    </button>
                                )}
                            </div>
                        </>
                    )}
                </div>
            )}
            
            {!collapsed && (
                <div className={clsx(contentClasses, "overflow-auto")} ref={activeTab === 'output' ? outputContainer : undefined}>
                    {activeTab === 'output' ? (
                        <pre className="text-xs text-white whitespace-pre font-mono">
                            {processedOutput}
                        </pre>
                    ) : (
                        <VMOutput outputRef={outputContainer} isActive={activeTab === 'vm'} />
                    )}
                </div>
            )}
        </div>
    );
}