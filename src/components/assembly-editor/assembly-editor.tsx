import { useEffect, useState, useCallback, useMemo } from "react";
import { Editor } from "../editor/editor.tsx";
import { editorManager } from "../../services/editor-manager.service.ts";
import { AssemblyTokenizer } from "../editor/services/assembly-tokenizer.ts";
import { CpuChipIcon, DocumentTextIcon, ArrowPathIcon } from "@heroicons/react/24/solid";
import { IconButton } from "../ui/icon-button.tsx";
import { settingsStore } from "../../stores/settings.store.ts";
import { useStoreSubscribe } from "../../hooks/use-store-subscribe.tsx";
import { assemblyOutputStore } from "../../stores/assembly-output.store.ts";
import { AssemblyOutput } from "./assembly-output.tsx";
import { DraggableVSep } from "../ui/draggable-vsep.tsx";
import { useLocalStorageState } from "../../hooks/use-local-storage-state.tsx";
import { EditorStore } from "../editor/stores/editor.store.ts";
import { createAssembler } from "../../services/ripple-assembler/index.ts";
import { AssemblyQuickNavStore, type AssemblyNavigationItem } from "./stores/assembly-quick-nav.store.ts";
import { AssemblyQuickNav } from "./components/assembly-quick-nav.tsx";
import {HSep} from "../helper-components.tsx";

export function AssemblyEditor() {
    const [assemblyEditor, setAssemblyEditor] = useState<EditorStore | null>(null);
    const [showOutput, setShowOutput] = useLocalStorageState("assemblyShowOutput", false);
    const [leftPanelWidth, setLeftPanelWidth] = useLocalStorageState("assemblyLeftPanelWidth", 60);
    const settings = useStoreSubscribe(settingsStore.settings);
    const autoCompile = settings?.assembly?.autoCompile ?? false;
    const autoOpenOutput = settings?.assembly?.autoOpenOutput ?? true;
    
    // Subscribe to minimap state
    const [minimapEnabled, setMinimapEnabled] = useLocalStorageState("assemblyMinimap", false);
    
    // Create quick nav store
    const quickNavStore = useMemo(() => new AssemblyQuickNavStore(), []);
    
    useEffect(() => {
        if (assemblyEditor) {
            const sub = assemblyEditor.showMinimap.subscribe(setMinimapEnabled);
            return () => sub.unsubscribe();
        }
    }, [assemblyEditor]);
    
    // Extract navigation items (labels and marks) from the code
    const extractNavigationItems = useCallback((): AssemblyNavigationItem[] => {
        if (!assemblyEditor) return [];
        
        const lines = assemblyEditor.editorState.getValue().lines;
        const items: AssemblyNavigationItem[] = [];
        
        lines.forEach((line, lineIndex) => {
            // Extract labels
            const labelMatch = line.text.match(/^([a-zA-Z_][a-zA-Z0-9_]*):/);
            if (labelMatch) {
                items.push({
                    type: 'label',
                    name: labelMatch[1],
                    line: lineIndex,
                    column: 0
                });
            }
            
            // Extract mark comments (// MARK:)
            const markMatch = line.text.match(/\/\/\s*MARK:\s*(.+)/);
            if (markMatch) {
                items.push({
                    type: 'mark',
                    name: markMatch[1].trim(),
                    line: lineIndex,
                    column: line.text.indexOf('// MARK:')
                });
            }
        });
        
        return items;
    }, [assemblyEditor]);
    
    // Handle keyboard shortcuts
    useEffect(() => {
        if (!assemblyEditor) return;
        
        const handleKeyDown = (e: KeyboardEvent) => {
            // Cmd+P for quick navigation
            if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
                e.preventDefault();
                const items = extractNavigationItems();
                quickNavStore.show(items);
            }
        };
        
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [assemblyEditor, quickNavStore, extractNavigationItems]);

    useEffect(() => {
        // Create assembly editor on mount
        const editor = editorManager.createEditor({
            id: 'assembly',
            tokenizer: new AssemblyTokenizer(),
            mode: 'insert',
            settings: {
                showDebug: false,
                showMinimap: minimapEnabled
            },
            initialContent: `; RippleVM Assembly Editor
; Use the Assemble button to compile your code

.data
    ; Define your data section here
    message: .asciiz "Hello, RippleVM!\\n"

.code
start:
    ; Your code starts here
    LI R3, 0        ; Initialize counter
    
main_loop:
    ; Load and print message character
    LOAD R4, R3, message
    BEQ R4, R0, done    ; Exit if null terminator
    
    ; Output character
    STORE R4, R0, 0     ; Store to I/O address
    
    ; Increment counter
    ADDI R3, R3, 1
    
    ; Continue loop
    JAL R0, main_loop
    
done:
    HALT
`
        });
        setAssemblyEditor(editor);

        // Cleanup on unmount
        return () => {
            editorManager.destroyEditor('assembly');
        };
    }, [minimapEnabled]);

    // Function to assemble code
    const assembleCode = useCallback(async () => {
        if (!assemblyEditor) return;

        const code = assemblyEditor.getText();
        
        try {
            // Create assembler with current settings
            const assembler = createAssembler({
                bankSize: settings?.assembly?.bankSize,
                maxImmediate: settings?.assembly?.maxImmediate
            });
            
            // Use the configured assembler
            const result = assembler.assemble(code);
            
            if (result.errors.length > 0) {
                // Report errors
                assemblyOutputStore.setError(result.errors.join('\n'));
                
                // TODO: Add inline error reporting
                console.error('Assembly errors:', result.errors);
            } else {
                // Set successful output
                assemblyOutputStore.setOutput({
                    instructions: result.instructions,
                    labels: result.labels,
                    dataLabels: result.dataLabels,
                    memoryData: result.memoryData
                });
                
                // Auto-open output panel if configured
                if (autoOpenOutput && !showOutput) {
                    setShowOutput(true);
                }
            }
        } catch (error) {
            assemblyOutputStore.setError(`Assembly failed: ${error}`);
            console.error('Assembly error:', error);
        }
    }, [assemblyEditor, autoOpenOutput, showOutput, setShowOutput, settings?.assembly?.bankSize, settings?.assembly?.maxImmediate]);

    // Auto-compile effect
    useEffect(() => {
        if (!autoCompile || !assemblyEditor) return;

        let timeoutId: number;

        // Subscribe to editor changes
        const subscription = assemblyEditor.editorState.subscribe(() => {
            // Clear previous timeout
            clearTimeout(timeoutId);

            // Debounce the compilation
            timeoutId = setTimeout(() => {
                assembleCode();
            }, 500); // 500ms delay for more responsive feedback
        });

        // Initial compilation
        assembleCode();

        return () => {
            clearTimeout(timeoutId);
            subscription.unsubscribe();
        };
    }, [autoCompile, assemblyEditor, assembleCode]);

    const handleResize = useCallback((leftWidth: number) => {
        const container = document.querySelector('.assembly-editor-container');
        if (container) {
            const containerWidth = container.clientWidth;
            const percentage = (leftWidth / containerWidth) * 100;
            setLeftPanelWidth(Math.max(20, Math.min(80, percentage)));
        }
    }, [setLeftPanelWidth]);

    // Handle jump to label
    const handleJumpToLabel = useCallback((labelName: string) => {
        if (!assemblyEditor) return;
        
        const lines = assemblyEditor.getText().split('\n');
        for (let i = 0; i < lines.length; i++) {
            const labelMatch = lines[i].match(/^([a-zA-Z_][a-zA-Z0-9_]*):/);
            if (labelMatch && labelMatch[1] === labelName) {
                // Set navigation flag for center scrolling
                assemblyEditor.isNavigating.next(true);
                // Jump to the line with the label
                assemblyEditor.setCursorPosition({ line: i, column: 0 });
                // Focus the editor
                assemblyEditor.focus();
                break;
            }
        }
    }, [assemblyEditor]);
    
    // Handle quick navigation
    const handleQuickNavigate = useCallback((item: AssemblyNavigationItem) => {
        if (!assemblyEditor) return;
        
        // Set navigation flag for center scrolling
        assemblyEditor.isNavigating.next(true);
        // Jump to the item
        assemblyEditor.setCursorPosition({ line: item.line, column: item.column });
        // Focus the editor
        assemblyEditor.focus();
    }, [assemblyEditor]);

    if (!assemblyEditor) {
        return <div className="v grow-1 bg-zinc-950">Loading...</div>;
    }

    return (
        <div className="v grow-1 bg-zinc-950">
            <HSep/>
            <div className="h grow-1 relative assembly-editor-container">
                <div 
                    className="v grow-0 shrink-0 bg-zinc-950"
                    style={{ width: showOutput ? `${leftPanelWidth}%` : '100%' }}
                >
                    <div className="h items-center bg-zinc-900 text-zinc-500 text-xs font-bold p-2 min-h-8 border-b border-zinc-800">
                        <span className="mr-4">Assembly Editor</span>

                        <div className="w-px h-6 bg-zinc-700 mx-1"/>

                        <IconButton
                            icon={CpuChipIcon}
                            label="Assemble"
                            onClick={assembleCode}
                        />

                        <div className="w-px h-6 bg-zinc-700 mx-1"/>

                        <IconButton
                            icon={DocumentTextIcon}
                            label="Toggle Minimap"
                            onClick={() => {
                                const newValue = !minimapEnabled;
                                setMinimapEnabled(newValue);
                                assemblyEditor?.showMinimap.next(newValue);
                            }}
                            variant={minimapEnabled ? "info" : "default"}
                        />

                        <div className="w-px h-6 bg-zinc-700 mx-1"/>

                        <IconButton
                            icon={ArrowPathIcon}
                            label={autoCompile ? "Auto-compile On" : "Auto-compile Off"}
                            onClick={() => settingsStore.setAssemblyAutoCompile(!autoCompile)}
                            variant={autoCompile ? "info" : "default"}
                        />

                        <div className="ml-auto h gap-2">
                            <button
                                className="text-zinc-600 hover:text-zinc-400"
                                onClick={() => setShowOutput(!showOutput)}
                            >
                                {showOutput ? 'Hide Output' : 'Show Output'}
                            </button>
                        </div>
                    </div>
                    <Editor
                        store={assemblyEditor}
                        onFocus={() => editorManager.setActiveEditor('assembly')}
                    />
                </div>
                {showOutput && (
                    <>
                        <DraggableVSep onResize={handleResize} />
                        <div className="v grow-1 bg-zinc-950">
                            <AssemblyOutput onJumpToLabel={handleJumpToLabel} />
                        </div>
                    </>
                )}
            </div>
            
            {/* Quick Navigation Modal */}
            <AssemblyQuickNav
                quickNavStore={quickNavStore}
                onNavigate={handleQuickNavigate}
            />
        </div>
    );
}