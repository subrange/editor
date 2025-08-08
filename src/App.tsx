import {Editor} from "./components/editor/editor.tsx";
import {HSep, VSep} from "./components/helper-components.tsx";
import {keybindingsService} from "./services/keybindings.service.ts";
import {Debugger} from "./components/debugger/debugger-v2.tsx";
import {Output} from "./components/editor/components/output.tsx";
import {useLocalStorageState} from "./hooks/use-local-storage-state.tsx";
import {Toolbar} from "./components/debugger/toolbar.tsx";
import clsx from "clsx";
import {ChevronDownIcon, ChevronUpIcon} from "@heroicons/react/16/solid";
import {Sidebar} from "./components/sidebar/sidebar.tsx";
import {editorManager} from "./services/editor-manager.service.ts";
import {EditorStore} from "./components/editor/stores/editor.store.ts";
import {useEffect, useState, useCallback} from "react";
import {ProgressiveMacroTokenizer} from "./components/editor/services/macro-tokenizer-progressive.ts";
import {createAsyncMacroExpander} from "./services/macro-expander/create-macro-expander.ts";
import {CpuChipIcon, ArrowPathIcon, DocumentTextIcon, CommandLineIcon} from "@heroicons/react/24/solid";
import {IconButton} from "./components/ui/icon-button.tsx";

import {settingsStore} from "./stores/settings.store";
import {useStoreSubscribe} from "./hooks/use-store-subscribe";
import {WorkerTokenizer} from "./services/tokenizer/worker-tokenizer-adapter.ts";
import {interpreterStore} from "./components/debugger/interpreter.store.ts";
// import {MacroContextPanel} from "./components/debugger/macro-context-panel.tsx";
import {DraggableVSep} from "./components/ui/draggable-vsep.tsx";
import {outputStore} from "./stores/output.store.ts";
import {vmOutputService} from "./services/vm-output.service.ts";
import {AssemblyEditor} from "./components/assembly-editor/assembly-editor.tsx";

// Initialize VM output service
vmOutputService.initialize();

function EditorPanel() {
    const [mainEditor, setMainEditor] = useState<EditorStore | null>(null);
    const [macroEditor, setMacroEditor] = useState<EditorStore | null>(null);
    const [showMacroEditor, setShowMacroEditor] = useLocalStorageState("showMacroEditor", false);
    const [showMainEditor, setShowMainEditor] = useLocalStorageState("showMainEditor", true);
    const [leftPanelWidth, setLeftPanelWidth] = useLocalStorageState("editorLeftPanelWidth", 50); // percentage
    const settings = useStoreSubscribe(settingsStore.settings);
    const autoExpand = settings?.macro.autoExpand ?? false;
    const [macroExpander] = useState(() => createAsyncMacroExpander());
    
    // Subscribe to minimap states
    const [mainEditorMinimapEnabled, setMainEditorMinimapEnabled] = useState(false);
    const [macroEditorMinimapEnabled, setMacroEditorMinimapEnabled] = useState(true);
    
    useEffect(() => {
        if (mainEditor) {
            const sub = mainEditor.showMinimap.subscribe(setMainEditorMinimapEnabled);
            return () => sub.unsubscribe();
        }
    }, [mainEditor]);
    
    useEffect(() => {
        if (macroEditor) {
            const sub = macroEditor.showMinimap.subscribe(setMacroEditorMinimapEnabled);
            return () => sub.unsubscribe();
        }
    }, [macroEditor]);

    useEffect(() => {
        // Create main editor on mount
        const editor = editorManager.createEditor({
            id: 'main',
            tokenizer: new WorkerTokenizer(() => {
                console.log("retokenized")
                editor.editorState.next({...editor.editorState.value});
            }),
            // tokenizer: new DummyTokenizer(),
            mode: 'insert',
            settings: {
                showDebug: true,
                showMinimap: false  // Main editor: minimap off by default
            },
        });
        setMainEditor(editor);

        // Create macro editor if needed
        if (showMacroEditor) {
            const macro = editorManager.createEditor({
                id: 'macro',
                tokenizer: new ProgressiveMacroTokenizer(),
                mode: 'insert',
                settings: {
                    showDebug: false,
                    showMinimap: true  // Macro editor: minimap on by default
                },
                initialContent: '#define clear [-]\n#define inc(n) {repeat(n, +)}\n#define dec(n) {repeat(n, -)}\n\n// Example usage:\n// @inc(5) @clear\n'
            });
            setMacroEditor(macro);
        }

        // Cleanup on unmount
        return () => {
            editorManager.destroyEditor('main');
            if (showMacroEditor) {
                editorManager.destroyEditor('macro');
            }
            macroExpander.destroy();
        };
    }, [showMacroEditor, macroExpander]);

    // Function to expand macros
    const expandMacros = useCallback(async () => {
        if (!macroEditor || !mainEditor) return;

        const macroCode = macroEditor.getText();
        const result = await macroExpander.expand(macroCode, {
            stripComments: settings?.macro.stripComments ?? true,
            collapseEmptyLines: settings?.macro.collapseEmptyLines ?? true,
            generateSourceMap: true // Enable source maps with V3 performance optimizations
        });

        if (result.errors.length > 0) {
            // In auto mode, don't show alerts, just log
            if (!autoExpand) {
                console.error('Macro expansion errors:', result.errors);
                alert(`Macro expansion failed: ${result.errors[0].message}`);
            }
        } else {
            // Set expanded code to main editor
            mainEditor.setContent(result.expanded);

            // Set source map in interpreter if available
            if (result.sourceMap) {
                interpreterStore.setSourceMap(result.sourceMap);
            } else {
                // Clear source map if not generated
                interpreterStore.setSourceMap(undefined);
            }

            if (!autoExpand) {
                console.log('Macros expanded successfully');
            }
        }
    }, [macroEditor, mainEditor, settings, autoExpand, macroExpander]);

    // Auto-expand effect
    useEffect(() => {
        // if (!autoExpand || !macroEditor || !mainEditor) return;
        //
        // let timeoutId: number;
        //
        // // Subscribe to macro editor changes
        // const subscription = macroEditor.editorState.subscribe(() => {
        //     // Clear previous timeout
        //     clearTimeout(timeoutId);
        //
        //     // Debounce the expansion to avoid too frequent updates
        //     timeoutId = setTimeout(() => {
        //         //expandMacros();
        //     }, 500); // 500ms delay
        // });
        //
        // // Initial expansion
        // expandMacros();
        //
        // return () => {
        //     clearTimeout(timeoutId);
        //     subscription.unsubscribe();
        // };
        if (!autoExpand || !macroEditor || !mainEditor) return;
        const tokenizer = macroEditor.getTokenizer();

        // Subscribe to tokenizer state changes if it's an enhanced macro tokenizer
        // useEffect(() => {
        if (tokenizer instanceof ProgressiveMacroTokenizer) {
            // console.log('Subscribing to tokenizer state changes');
            const unsubscribe = tokenizer.onStateChange((state) => {
                // console.log('Tokenizer state changed, forcing re-render');
                // // Force re-render by updating version
                // setMacroExpansionVersion(v => v + 1);

                if (!state) return;

                if (state.expanderErrors.length > 0) {
                    // In auto mode, don't show alerts, just log
                    if (!autoExpand) {
                        console.error('Macro expansion errors:', state.expanderErrors);
                    }
                } else {
                    // Set expanded code to main editor
                    mainEditor.setContent(state.expanded);

                    // Set source map in interpreter if available
                    if (state.sourceMap) {
                        interpreterStore.setSourceMap(state.sourceMap);
                        console.log(`Source map updated with ${state.sourceMap.entries.length} entries`);
                    } else {
                        interpreterStore.setSourceMap(undefined);
                    }

                    if (!autoExpand) {
                        console.log('Macros expanded successfully');
                    }
                }
            });
            return unsubscribe;
        }
        // }, [tokenizer]);
    }, [autoExpand, macroEditor, mainEditor, settings]);

    const handleResize = useCallback((leftWidth: number) => {
        const container = document.querySelector('.editor-panel-container');
        if (container) {
            const containerWidth = container.clientWidth;
            const percentage = (leftWidth / containerWidth) * 100;
            setLeftPanelWidth(Math.max(20, Math.min(80, percentage))); // Clamp between 20% and 80%
        }
    }, [setLeftPanelWidth]);

    if (!mainEditor) {
        return <div className="v grow-1 bg-zinc-950">Loading...</div>;
    }

    return <div className="h grow-1 relative editor-panel-container">
        {showMacroEditor && macroEditor && (
            <>
                <div 
                    className="v grow-0 shrink-0 bg-zinc-950"
                    style={{ width: showMainEditor ? `${leftPanelWidth}%` : '100%' }}
                >
                    <div
                        className="h items-center bg-zinc-900 text-zinc-500 text-xs font-bold p-2 min-h-8 border-b border-zinc-800">
                        <span className="mr-4">Macro Editor</span>

                        <div className="w-px h-6 bg-zinc-700 mx-1"/>

                        <IconButton
                            icon={CpuChipIcon}
                            label="Expand Macros"
                            onClick={expandMacros}
                        />

                        <div className="w-px h-6 bg-zinc-700 mx-1"/>

                        <IconButton
                            icon={ArrowPathIcon}
                            label={autoExpand ? "Auto-expand On" : "Auto-expand Off"}
                            onClick={() => settingsStore.setMacroAutoExpand(!autoExpand)}
                            variant={autoExpand ? "info" : "default"}
                        />

                        <div className="w-px h-6 bg-zinc-700 mx-1"/>

                        <IconButton
                            icon={DocumentTextIcon}
                            label="Toggle Minimap"
                            onClick={() => macroEditor?.showMinimap.next(!macroEditorMinimapEnabled)}
                            variant={macroEditorMinimapEnabled ? "info" : "default"}
                        />

                        {!showMainEditor && (
                            <>
                                <div className="w-px h-6 bg-zinc-700 mx-1"/>
                                <button
                                    className="text-zinc-600 hover:text-zinc-400"
                                    onClick={() => setShowMainEditor(true)}
                                >
                                    Show Main Editor
                                </button>
                            </>
                        )}

                        <button
                            className="ml-auto text-zinc-600 hover:text-zinc-400"
                            onClick={() => {
                                if (showMainEditor) {
                                    setShowMacroEditor(false);
                                }
                            }}
                            disabled={!showMainEditor}
                        >
                            ✕
                        </button>
                    </div>
                    <Editor
                        store={macroEditor}
                        onFocus={() => editorManager.setActiveEditor('macro')}
                    />
                </div>
                {showMainEditor && <DraggableVSep onResize={handleResize} />}
            </>
        )}
        {showMainEditor && (
            <div className="v grow-1 bg-zinc-950">
                <div className="h items-center bg-zinc-900 text-zinc-500 text-xs font-bold p-2 min-h-8 border-b border-zinc-800">
                    Main Editor
                    
                    <div className="w-px h-6 bg-zinc-700 mx-1"/>
                    
                    <IconButton
                        icon={DocumentTextIcon}
                        label="Toggle Minimap"
                        onClick={() => mainEditor?.showMinimap.next(!mainEditorMinimapEnabled)}
                        variant={mainEditorMinimapEnabled ? "info" : "default"}
                    />
                    
                    <div className="ml-auto h gap-2">
                        {!showMacroEditor && (
                            <button
                                className="text-zinc-600 hover:text-zinc-400"
                                onClick={() => setShowMacroEditor(true)}
                            >
                                Show Macro Editor
                            </button>
                        )}
                        <button
                            className="text-zinc-600 hover:text-zinc-400 disabled:text-zinc-800"
                            onClick={() => {
                                if (showMacroEditor) {
                                    setShowMainEditor(false);
                                }
                            }}
                            disabled={!showMacroEditor}
                        >
                            ✕
                        </button>
                    </div>
                </div>
                <Editor
                    store={mainEditor}
                    onFocus={() => editorManager.setActiveEditor('main')}
                />
            </div>
        )}
        {!showMainEditor && !showMacroEditor && (
            <div className="v grow-1 items-center justify-center bg-zinc-950 text-zinc-600">
                <p className="mb-4">No editors visible</p>
                <div className="h gap-4">
                    <button
                        className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded"
                        onClick={() => setShowMainEditor(true)}
                    >
                        Show Main Editor
                    </button>
                    <button
                        className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded"
                        onClick={() => setShowMacroEditor(true)}
                    >
                        Show Macro Editor
                    </button>
                </div>
            </div>
        )}
    </div>;
}

function DebugPanel() {
    const [collapsed, setCollapsed] = useLocalStorageState("debugCollapsed", true);
    const settings = useStoreSubscribe(settingsStore.settings);
    const viewMode = settings?.debugger.viewMode ?? 'normal';

    return <div className={clsx("v bg-zinc-900 transition-all", {
        "h-96 min-h-96": !collapsed && viewMode === 'lane',
        "h-64 min-h-64": !collapsed && viewMode === 'normal',
        "h-36 min-h-36": !collapsed && viewMode === 'compact',
        "h-8 min-h-8": collapsed,
    })}>
        <button className={clsx(
            "h bg-zinc-900 text-zinc-500 text-xs font-bold p-2 h-8 min-h-8 border-t border-zinc-800",
            "hover:bg-zinc-800 hover:text-zinc-400 transition-colors",
            "gap-2"
        )}
                onClick={() => setCollapsed(!collapsed)}
        >
            {
                collapsed
                    ? <ChevronDownIcon/>
                    : <ChevronUpIcon/>
            }
            Debugger
        </button>
        {
            !collapsed && (
                <><HSep/>

                <div className="h h-full">
                    <div className="v h-full grow">
                        <Debugger/>
                    </div>
                </div></>
            )
        }

    </div>;
}

function WorkspacePanel() {
    const [activeEditor, setActiveEditor] = useLocalStorageState<'brainfuck' | 'assembly'>('activeEditorType', 'brainfuck');
    
    return <div className="v grow-1 bg-zinc-950">
        {/* Editor selector tabs */}
        <div className="h items-center bg-zinc-900 text-zinc-500 text-xs font-bold px-2 min-h-8 border-b border-zinc-800">
            <div className="h gap-0">
                <button
                    className={clsx(
                        "px-3 py-2 text-xs font-bold transition-colors",
                        activeEditor === 'brainfuck' 
                            ? "text-zinc-400 bg-zinc-800" 
                            : "text-zinc-600 hover:text-zinc-500 hover:bg-zinc-800/50"
                    )}
                    onClick={() => setActiveEditor('brainfuck')}
                >
                    <CpuChipIcon className="w-3 h-3 inline mr-1" />
                    Brainfuck
                </button>
                <button
                    className={clsx(
                        "px-3 py-2 text-xs font-bold transition-colors",
                        activeEditor === 'assembly' 
                            ? "text-zinc-400 bg-zinc-800" 
                            : "text-zinc-600 hover:text-zinc-500 hover:bg-zinc-800/50"
                    )}
                    onClick={() => setActiveEditor('assembly')}
                >
                    <CommandLineIcon className="w-3 h-3 inline mr-1" />
                    Assembly
                </button>
            </div>
        </div>
        
        {/* Content based on active editor */}
        {activeEditor === 'brainfuck' ? (
            <>
                <DebugPanel/>
                <Toolbar/>
                <HSep/>
                <EditorPanel/>
            </>
        ) : (
            <>
                <Toolbar/>
                <AssemblyEditor />
            </>
        )}
    </div>;
}

export default function App() {
    const outputState = useStoreSubscribe(outputStore.state);
    const { position, collapsed, width } = outputState;

    const handleOutputResize = useCallback((newWidth: number) => {
        outputStore.setSize('width', newWidth);
    }, []);

    if (position === 'right' && !collapsed) {
        // Right layout with output panel beside entire workspace
        return (
            <div className="h grow-1 outline-0 app-container" tabIndex={0}
                 onKeyDownCapture={e => keybindingsService.handleKeyEvent(e.nativeEvent)}>
                <div className="sidebar">
                    <Sidebar/>
                </div>
                <VSep/>
                
                <div className="h grow-1">
                    <WorkspacePanel/>
                </div>
                <DraggableVSep 
                    onResize={(leftWidth) => {
                        // leftWidth is the distance from parent's left edge to separator
                        // Since parent is app-container, leftWidth includes sidebar + workspace
                        // We need to calculate the output panel width
                        const appContainer = document.querySelector('.app-container');
                        if (appContainer) {
                            const totalWidth = appContainer.clientWidth;
                            // Output width is simply the remaining space after leftWidth
                            const newOutputWidth = totalWidth - leftWidth;
                            handleOutputResize(Math.max(200, Math.min(800, newOutputWidth)));
                        }
                    }}
                    minLeftWidth={400}
                    minRightWidth={200}
                />
                <div className="h" style={{ width: `${width}px`, flexShrink: 0 }}>
                    <Output position="right" />
                </div>
            </div>
        );
    }

    // Default layout - output at bottom or collapsed
    return (
        <div className="h grow-1 outline-0 app-container" tabIndex={0}
             onKeyDownCapture={e => keybindingsService.handleKeyEvent(e.nativeEvent)}>
            <div className="sidebar">
                <Sidebar/>
            </div>
            <VSep/>
            
            {position === 'right' && collapsed ? (
                // Collapsed right layout - proper flex structure
                <>
                    <div className="v grow-1">
                        <WorkspacePanel/>
                    </div>
                    <VSep/>
                    <div className="h" style={{ width: '32px', flexShrink: 0 }}>
                        <Output position="right" />
                    </div>
                </>
            ) : (
                // Bottom layout or floating
                <div className="v grow-1 relative">
                    <WorkspacePanel/>
                    
                    {/* Output panel - bottom position */}
                    {position === 'bottom' && <Output position="bottom" />}
                    
                    {/* Floating position */}
                    {position === 'floating' && (
                        <Output 
                            position="floating" 
                            onClose={() => outputStore.setCollapsed(true)} 
                        />
                    )}
                </div>
            )}
        </div>
    );
}
