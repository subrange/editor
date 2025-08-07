import { useRef, useLayoutEffect, useState } from "react";
import { useStoreSubscribe } from "../../../hooks/use-store-subscribe";
import { vmTerminalStore } from "../../../stores/vm-terminal.store";
import { CogIcon, XMarkIcon } from "@heroicons/react/24/outline";
import clsx from "clsx";

interface VMOutputProps {
    outputRef?: React.RefObject<HTMLDivElement | null>;
    isActive?: boolean;
}

export function VMOutput({ outputRef, isActive = true }: VMOutputProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const activeRef = outputRef || containerRef;
    const [showConfig, setShowConfig] = useState(false);
    const [tempOutCell, setTempOutCell] = useState<string>("");
    const [tempFlagCell, setTempFlagCell] = useState<string>("");
    
    const vmTerminalState = useStoreSubscribe(vmTerminalStore.state);
    
    const { config, output } = vmTerminalState;
    const { outCellIndex, outFlagCellIndex, clearOnRead, enabled } = config;

    // Auto-scroll to bottom when content changes
    useLayoutEffect(() => {
        setTimeout(() => {
            if (activeRef.current) {
                activeRef.current.scrollTop = activeRef.current.scrollHeight;
            }
        }, 10);
    }, [output]);
    
    const handleConfigSubmit = () => {
        const outCell = parseInt(tempOutCell);
        const flagCell = parseInt(tempFlagCell);
        
        if (!isNaN(outCell) && outCell >= 0) {
            vmTerminalStore.updateConfig({ outCellIndex: outCell });
        }
        if (!isNaN(flagCell) && flagCell >= 0) {
            vmTerminalStore.updateConfig({ outFlagCellIndex: flagCell });
        }
        
        setTempOutCell("");
        setTempFlagCell("");
        setShowConfig(false);
    };

    return (
        <div ref={activeRef} className="flex flex-col h-full overflow-auto bg-zinc-950">
            {/* Header */}
            <div className="sticky top-0 z-10 bg-zinc-900 border-b border-zinc-800">
                <div className="px-3 py-2 flex items-center justify-between gap-2">
                    <div className="flex items-center gap-2 min-w-0 flex-1">
                        <button
                            onClick={() => vmTerminalStore.updateConfig({ enabled: !enabled })}
                            className={clsx(
                                "px-2 py-0.5 text-xs font-medium rounded transition-all whitespace-nowrap",
                                enabled 
                                    ? "bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30" 
                                    : "bg-zinc-800 text-zinc-500 hover:bg-zinc-700 hover:text-zinc-400"
                            )}
                            title={enabled ? 'VM output is syncing' : 'VM output sync is disabled'}
                        >
                            {enabled ? 'ON' : 'OFF'}
                        </button>
                        <div className="flex items-center gap-2 text-[11px] text-zinc-500 font-mono overflow-hidden">
                            <span className="whitespace-nowrap" title={`Output cell: ${outCellIndex}`}>
                                [{outCellIndex}]
                            </span>
                            <span className="text-zinc-700">→</span>
                            <span className="whitespace-nowrap" title={`Flag cell: ${outFlagCellIndex}`}>
                                [{outFlagCellIndex}]
                            </span>
                        </div>
                    </div>
                    <button
                        onClick={() => setShowConfig(!showConfig)}
                        className={clsx(
                            "p-1 rounded transition-all flex-shrink-0",
                            showConfig 
                                ? "bg-zinc-800 text-zinc-400" 
                                : "hover:bg-zinc-800 text-zinc-500 hover:text-zinc-400"
                        )}
                        title="Configure VM output settings"
                    >
                        <CogIcon className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>
            
            {/* Configuration Panel */}
            {showConfig && (
                <div className="border-b border-zinc-800 bg-zinc-900/50">
                    <div className="p-4 space-y-4">
                        <div className="space-y-3">
                            {/* Output Cell Index */}
                            <div className="space-y-2">
                                <label className="text-sm font-medium text-zinc-300">
                                    Output Cell Index
                                </label>
                                <input
                                    type="number"
                                    min="0"
                                    value={tempOutCell || outCellIndex}
                                    onChange={(e) => setTempOutCell(e.target.value)}
                                    onKeyDown={(e) => e.key === 'Enter' && handleConfigSubmit()}
                                    className="w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                                    placeholder={String(outCellIndex)}
                                />
                                <p className="text-xs text-zinc-500">
                                    Cell that contains the character code to output
                                </p>
                            </div>

                            {/* Flag Cell Index */}
                            <div className="space-y-2">
                                <label className="text-sm font-medium text-zinc-300">
                                    Flag Cell Index
                                </label>
                                <input
                                    type="number"
                                    min="0"
                                    value={tempFlagCell || outFlagCellIndex}
                                    onChange={(e) => setTempFlagCell(e.target.value)}
                                    onKeyDown={(e) => e.key === 'Enter' && handleConfigSubmit()}
                                    className="w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                                    placeholder={String(outFlagCellIndex)}
                                />
                                <p className="text-xs text-zinc-500">
                                    Cell that triggers output when set to 1
                                </p>
                            </div>

                            {/* Clear on Read Checkbox */}
                            <label className="flex items-center gap-3 cursor-pointer group pt-2">
                                <input
                                    type="checkbox"
                                    id="clearOnRead"
                                    checked={clearOnRead}
                                    onChange={(e) => vmTerminalStore.updateConfig({ clearOnRead: e.target.checked })}
                                    className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                                />
                                <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                                    Clear flag after reading
                                </span>
                            </label>
                        </div>

                        {/* Action Buttons */}
                        <div className="flex items-center justify-between pt-2 border-t border-zinc-800">
                            <button
                                onClick={() => {
                                    vmTerminalStore.resetConfig();
                                    setTempOutCell("");
                                    setTempFlagCell("");
                                }}
                                className="px-3 py-1.5 text-sm text-zinc-400 hover:text-zinc-300 border border-zinc-700 hover:border-zinc-600 rounded transition-all"
                            >
                                Reset to Defaults
                            </button>
                            <div className="flex items-center gap-2">
                                <button
                                    onClick={() => {
                                        setShowConfig(false);
                                        setTempOutCell("");
                                        setTempFlagCell("");
                                    }}
                                    className="px-3 py-1.5 text-sm text-zinc-400 hover:text-zinc-300 transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    onClick={handleConfigSubmit}
                                    className="px-4 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 text-white font-medium rounded transition-colors"
                                >
                                    Apply Changes
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            )}
            
            {/* Output Content */}
            <div className="flex-1 overflow-auto">
                <pre className="text-sm text-zinc-100 whitespace-pre-wrap font-mono p-4 min-h-full">
                    {output || (
                        <div className="text-zinc-500 italic">
                            {enabled ? (
                                <>
                                    <p>No output yet.</p>
                                    <p className="text-xs mt-2 text-zinc-600">
                                        VM will output when cell[{outFlagCellIndex}] = 1
                                    </p>
                                </>
                            ) : (
                                <>
                                    <p>VM output sync is disabled.</p>
                                    <p className="text-xs mt-2 text-zinc-600">
                                        Enable sync to see output from the virtual machine.
                                    </p>
                                </>
                            )}
                        </div>
                    )}
                </pre>
            </div>
            
            {/* Footer with Clear Button */}
            {output && (
                <div className="sticky bottom-0 bg-zinc-900 border-t border-zinc-800 px-4 py-2">
                    <div className="flex items-center justify-between">
                        <span className="text-xs text-zinc-500">
                            {output.length} characters
                        </span>
                        <button
                            onClick={() => vmTerminalStore.clearOutput()}
                            className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-zinc-800 hover:bg-zinc-700 text-zinc-400 hover:text-zinc-300 rounded transition-all"
                        >
                            <XMarkIcon className="w-3.5 h-3.5" />
                            Clear Output
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}