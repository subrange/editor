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
        <div className="bg-zinc-900 border-t border-zinc-800 p-2 text-xs">
            <div className="text-zinc-500 font-bold mb-1">Source Map Debug Info</div>
            
            <div className="text-zinc-400 space-y-1">
                <div>Has source map: {hasSourceMap ? 'Yes' : 'No'}</div>
                {hasSourceMap && state.sourceMap && (
                    <div>Source map entries: {state.sourceMap.entries.length}</div>
                )}
                <div>Current source position: {currentSourcePosition ? 
                    `Line ${currentSourcePosition.line + 1}, Col ${currentSourcePosition.column + 1}` : 
                    'None'}</div>
                <div>Macro context: {macroContext ? `${macroContext.length} levels` : 'None'}</div>
                {macroContext && macroContext.length > 0 && (
                    <div className="mt-2">
                        <div className="text-zinc-500 text-xs">Call Stack:</div>
                        {macroContext.map((ctx, i) => (
                            <div key={i} className="ml-2">
                                @{ctx.macroName}
                                {ctx.parameters && Object.keys(ctx.parameters).length > 0 && 
                                    ` (${Object.entries(ctx.parameters).map(([k,v]) => `${k}=${v}`).join(', ')})`
                                }
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}