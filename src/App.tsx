import {Editor} from "./components/editor/editor.tsx";
import {HSep, VSep} from "./components/helper-components.tsx";
import {keybindingsService} from "./services/keybindings.service.ts";
import {Debugger} from "./components/debugger/debugger.tsx";
import {Output} from "./components/editor/output.tsx";
import {useLocalStorageState} from "./hooks/use-local-storage-state.tsx";
import {Toolbar} from "./components/debugger/toolbar.tsx";
import clsx from "clsx";
import {ChevronDownIcon, ChevronUpIcon} from "@heroicons/react/16/solid";
import {Sidebar} from "./components/sidebar/sidebar.tsx";
import {editorManager} from "./services/editor-manager.service.ts";
import {EditorStore} from "./components/editor/editor.store.ts";
import {useEffect, useState} from "react";
import {MacroTokenizer} from "./components/editor/macro-tokenizer.ts";



function EditorPanel() {
    const [mainEditor, setMainEditor] = useState<EditorStore | null>(null);
    const [macroEditor, setMacroEditor] = useState<EditorStore | null>(null);
    const [showMacroEditor, setShowMacroEditor] = useLocalStorageState("showMacroEditor", false);
    
    useEffect(() => {
        // Create main editor on mount
        const editor = editorManager.createEditor({
            id: 'main',
            mode: 'insert',
            settings: {
                showDebug: true
            },
        });
        setMainEditor(editor);
        
        // Create macro editor if needed
        if (showMacroEditor) {
            const macro = editorManager.createEditor({
                id: 'macro',
                tokenizer: new MacroTokenizer(),
                mode: 'insert',
                settings: {
                    showDebug: false
                },
                initialContent: '// Macro definitions\n// Example: @multiply($n) = [>+<-]$n\n\n'
            });
            setMacroEditor(macro);
        }
        
        // Cleanup on unmount
        return () => {
            editorManager.destroyEditor('main');
            if (showMacroEditor) {
                editorManager.destroyEditor('macro');
            }
        };
    }, [showMacroEditor]);
    
    if (!mainEditor) {
        return <div className="v grow-1 bg-zinc-950">Loading...</div>;
    }
    
    return <div className="h grow-1 relative">
        {showMacroEditor && macroEditor && (
            <>
                <div className="v grow-1 min-w-1/2 bg-zinc-950">
                    <div className="h bg-zinc-900 text-zinc-500 text-xs font-bold p-2 min-h-8 border-b border-zinc-800">
                        Macro Editor
                        <button 
                            className="ml-auto text-zinc-600 hover:text-zinc-400"
                            onClick={() => setShowMacroEditor(false)}
                        >
                            ✕
                        </button>
                    </div>
                    <Editor 
                        store={macroEditor}
                        onFocus={() => editorManager.setActiveEditor('macro')}
                    />
                </div>
                <VSep/>
            </>
        )}
        <div className="v grow-1 min-w-1/2 bg-zinc-950">
            <div className="h bg-zinc-900 text-zinc-500 text-xs font-bold p-2 min-h-8 border-b border-zinc-800">
                Main Editor
                {!showMacroEditor && (
                    <button 
                        className="ml-auto text-zinc-600 hover:text-zinc-400"
                        onClick={() => setShowMacroEditor(true)}
                    >
                        Show Macro Editor
                    </button>
                )}
            </div>
            <Editor 
                store={mainEditor}
                onFocus={() => editorManager.setActiveEditor('main')}
            />
            <Output/>
        </div>
    </div>;
}

function DebugPanel() {
    const [collapsed, setCollapsed] = useLocalStorageState("debugCollapsed", true);

    return <div className={clsx("v bg-zinc-900 transition-all", {
        "h-64 min-h-64": !collapsed,
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
            Tape Viewer
        </button>
        {
            !collapsed && <Debugger />
        }

    </div>;
}

function WorkspacePanel() {
    return <div className="v grow-1 bg-zinc-950">
        <DebugPanel/>
        <Toolbar/>

        <HSep/>
        <EditorPanel/>
    </div>;
}

export default function App() {
  return (
    <div className="h grow-1 outline-0" tabIndex={0} onKeyDownCapture={e => keybindingsService.handleKeyEvent(e.nativeEvent)}>
        <Sidebar/>
        <VSep/>
        <WorkspacePanel/>
    </div>
  )
}
