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



function EditorPanel() {
    return <div className="v grow-1 bg-zinc-950">
        <Editor/>
        <Output/>
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
