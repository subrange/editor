import { useRef, useLayoutEffect, useEffect, useState } from "react";
import { useStoreSubscribe } from "../../../hooks/use-store-subscribe";
import { interpreterStore } from "../../debugger/interpreter-facade.store";
import { vmTerminalStore } from "../../../stores/vm-terminal.store";
import { settingsStore } from "../../../stores/settings.store";
import { CogIcon } from "@heroicons/react/24/outline";

interface VMOutputProps {
    outputRef?: React.RefObject<HTMLDivElement | null>;
}

export function VMOutput({ outputRef }: VMOutputProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const activeRef = outputRef || containerRef;
    const [showConfig, setShowConfig] = useState(false);
    const [tempOutCell, setTempOutCell] = useState<string>("");
    const [tempFlagCell, setTempFlagCell] = useState<string>("");
    
    const interpreterState = useStoreSubscribe(interpreterStore.state);
    const vmTerminalState = useStoreSubscribe(vmTerminalStore.state);
    const settings = useStoreSubscribe(settingsStore.settings);
    
    const { tape } = interpreterState;
    const { config, output } = vmTerminalState;
    const { outCellIndex, outFlagCellIndex, clearOnRead } = config;
    
    // Set up the VM output callback
    useEffect(() => {
        // Set the config first
        interpreterStore.setVMOutputConfig({ outCellIndex, outFlagCellIndex });
        
        const callback = (tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => {
            // Check if indices are valid
            if (tape.length <= Math.max(outCellIndex, outFlagCellIndex)) {
                return;
            }

            const charCode = tape[outCellIndex];
            const char = String.fromCharCode(charCode);
            vmTerminalStore.appendOutput(char);
        };
        
        // Register the callback
        interpreterStore.setVMOutputCallback(callback);
        
        // Cleanup on unmount or when indices change
        return () => {
            interpreterStore.setVMOutputCallback(null);
        };
    }, [outCellIndex, outFlagCellIndex]);

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
        <div ref={activeRef} className="flex flex-col h-full overflow-auto">
            <div className="p-2 bg-zinc-900 border-b border-zinc-800 flex items-center justify-between">
                <div className="flex gap-4 text-xs text-zinc-500">
                    <span>OUT: cell[{outCellIndex}]</span>
                    <span>FLAG: cell[{outFlagCellIndex}]</span>
                </div>
                <button
                    onClick={() => setShowConfig(!showConfig)}
                    className="p-1 hover:bg-zinc-800 rounded transition-colors"
                    title="Configure cell indices"
                >
                    <CogIcon className="w-4 h-4 text-zinc-500 hover:text-zinc-400" />
                </button>
            </div>
            
            {showConfig && (
                <div className="p-3 bg-zinc-800 border-b border-zinc-700">
                    <div className="space-y-2">
                        <div className="flex items-center gap-2">
                            <label className="text-xs text-zinc-400 w-20">OUT Cell:</label>
                            <input
                                type="number"
                                min="0"
                                value={tempOutCell || outCellIndex}
                                onChange={(e) => setTempOutCell(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && handleConfigSubmit()}
                                className="flex-1 px-2 py-1 text-xs bg-zinc-900 border border-zinc-700 rounded focus:border-zinc-600 focus:outline-none"
                                placeholder={String(outCellIndex)}
                            />
                        </div>
                        <div className="flex items-center gap-2">
                            <label className="text-xs text-zinc-400 w-20">FLAG Cell:</label>
                            <input
                                type="number"
                                min="0"
                                value={tempFlagCell || outFlagCellIndex}
                                onChange={(e) => setTempFlagCell(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && handleConfigSubmit()}
                                className="flex-1 px-2 py-1 text-xs bg-zinc-900 border border-zinc-700 rounded focus:border-zinc-600 focus:outline-none"
                                placeholder={String(outFlagCellIndex)}
                            />
                        </div>
                        <div className="flex items-center gap-2 mt-2">
                            <input
                                type="checkbox"
                                id="clearOnRead"
                                checked={clearOnRead}
                                onChange={(e) => vmTerminalStore.updateConfig({ clearOnRead: e.target.checked })}
                                className="w-3 h-3"
                            />
                            <label htmlFor="clearOnRead" className="text-xs text-zinc-400">
                                Clear flag after reading (BF program should handle this)
                            </label>
                        </div>
                        <div className="flex justify-end gap-2 mt-3">
                            <button
                                onClick={() => setShowConfig(false)}
                                className="text-xs px-2 py-1 text-zinc-500 hover:text-zinc-400"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleConfigSubmit}
                                className="text-xs px-3 py-1 bg-zinc-700 hover:bg-zinc-600 rounded text-zinc-300"
                            >
                                Apply
                            </button>
                        </div>
                    </div>
                </div>
            )}
            
            <pre className="text-xs text-white whitespace-pre-wrap font-mono p-4 flex-1">
                {output || <span className="text-zinc-600 italic">No output yet. VM will output when cell[{outFlagCellIndex}] = 1</span>}
            </pre>
            
            {output && (
                <div className="border-t border-zinc-800 p-2 flex justify-end">
                    <button
                        onClick={() => vmTerminalStore.clearOutput()}
                        className="text-xs px-2 py-1 bg-zinc-800 hover:bg-zinc-700 rounded text-zinc-500 hover:text-zinc-400 transition-colors"
                    >
                        Clear
                    </button>
                </div>
            )}
        </div>
    );
}