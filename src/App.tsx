import {Editor} from "./components/editor/editor.tsx";
import {HSep, VSep} from "./components/helper-components.tsx";
import {keybindingsService} from "./services/keybindings.service.ts";

function Sidebar() {
    return <div className="v w-80 h-screen bg-zinc-900">
        Hello, this is the left side!
    </div>;
}

function EditorPanel() {
    return <div className="v grow-1 bg-zinc-950">
        <Editor/>
    </div>;
}

function DebugPanel() {
    return <div className="h h-64 bg-zinc-900">
    </div>;
}

function WorkspacePanel() {
    return <div className="v grow-1 bg-zinc-950">
        <DebugPanel/>
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
