import { useStoreSubscribe } from "../../hooks/use-store-subscribe";
import { interpreterStore } from "./interpreter-facade.store";

export function MacroContextPanel() {
    const state = useStoreSubscribe(interpreterStore.state);
    const macroContext = state.macroContext;
    const currentSourcePosition = state.currentSourcePosition;
    const isRunning = state.isRunning;
    const hasSourceMap = state.sourceMap !== undefined;
    
    console.log("MacroContextPanel state:", {
        hasSourceMap,
        currentSourcePosition,
        macroContext,
        isRunning,
        sourceMap: state.sourceMap,
        sourceMapEntries: state.sourceMap?.entries?.length ?? 0,
        sampleEntry: state.sourceMap?.entries?.[0]
    });
    
    // Always show panel for debugging
    return (
        <div className="v bg-zinc-900 border-l border-zinc-800 min-w-[250px] max-w-[350px]">
            {/*<div className="h items-center bg-zinc-800 text-zinc-400 text-xs font-bold px-3 py-2 border-b border-zinc-700">*/}
            {/*    <span>Macro Context</span>*/}
            {/*</div>*/}
            
            <div className="v grow-1 overflow-y-auto">
                {/* Source Map Info Section */}
                <div className="border-b border-zinc-800 p-3">
                    <div className="text-zinc-500 text-xs font-semibold mb-2">Debug Info</div>
                    <div className="space-y-1">
                        <div className="h justify-between text-xs">
                            <span className="text-zinc-600">Source Map:</span>
                            <span className={hasSourceMap ? "text-green-500" : "text-zinc-500"}>
                                {hasSourceMap ? 'Active' : 'None'}
                            </span>
                        </div>
                        {hasSourceMap && state.sourceMap && (
                            <div className="h justify-between text-xs">
                                <span className="text-zinc-600">Entries:</span>
                                <span className="text-zinc-400">{state.sourceMap.entries.length}</span>
                            </div>
                        )}
                    </div>
                </div>
                
                {/* Current Position Section */}
                {currentSourcePosition && (
                    <div className="border-b border-zinc-800 p-3">
                        <div className="text-zinc-500 text-xs font-semibold mb-2">Current Position</div>
                        <div className="font-mono text-xs text-blue-400">
                            Line {currentSourcePosition.line + 1}, Col {currentSourcePosition.column + 1}
                        </div>
                    </div>
                )}
                
                {/* Macro Call Stack Section */}
                {macroContext && macroContext.length > 0 && (
                    <div className="p-3">
                        <div className="text-zinc-500 text-xs font-semibold mb-2">
                            Call Stack ({macroContext.length} {macroContext.length === 1 ? 'level' : 'levels'})
                        </div>
                        <div className="space-y-1">
                            {macroContext.map((ctx, i) => (
                                <div key={i} className="h items-start text-xs">
                                    <span className="text-zinc-600 mr-1 font-mono">{i}:</span>
                                    <div className="grow-1">
                                        <span className="text-green-400 font-mono">@{ctx.macroName}</span>
                                        {ctx.parameters && Object.keys(ctx.parameters).length > 0 && (
                                            <span className="text-zinc-400 ml-1">
                                                ({Object.entries(ctx.parameters).map(([k,v]) => 
                                                    <span key={k}>
                                                        <span className="text-zinc-500">{k}</span>
                                                        <span className="text-zinc-600">=</span>
                                                        <span className="text-yellow-400">{v}</span>
                                                    </span>
                                                ).reduce((prev, curr, i) => 
                                                    i === 0 ? [curr] : [...prev, <span key={`sep-${i}`} className="text-zinc-600">, </span>, curr]
                                                , [] as React.ReactNode[])})
                                            </span>
                                        )}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                )}
                
                {/* Empty State */}
                {(!macroContext || macroContext.length === 0) && !currentSourcePosition && (
                    <div className="p-3 text-center text-zinc-600 text-xs">
                        No macro context available
                    </div>
                )}
            </div>
        </div>
    );
}