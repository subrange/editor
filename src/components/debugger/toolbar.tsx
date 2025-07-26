import {
    PlayIcon,
    StopIcon,
    ArrowPathIcon,
    BoltIcon,
    ClockIcon, XMarkIcon,
    CursorArrowRaysIcon,
} from '@heroicons/react/24/solid';
import { interpreterStore } from "./interpreter-facade.store.ts";
import {useStoreSubscribe} from "../../hooks/use-store-subscribe.tsx";
import { useState } from 'react';
import {
    ForwardIcon,
    PauseIcon
} from '@heroicons/react/24/solid';
import {IconButton} from "../ui/icon-button.tsx";
import { editorManager } from "../../services/editor-manager.service.ts";

export function Toolbar() {
    const interpreterState = useStoreSubscribe(interpreterStore.state);
    const { isRunning, isPaused, isStopped } = interpreterState;
    const [delay, setDelay] = useState(50);
    const [showDelayInput, setShowDelayInput] = useState(false);

    const handleDelayChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const value = parseInt(e.target.value) || 0;
        setDelay(Math.max(0, Math.min(1000, value)));
    };

    const handleRunFromCursor = () => {
        const mainEditor = editorManager.getEditor('main');
        if (mainEditor) {
            const state = mainEditor.getState();
            const cursorPosition = state.selection.focus;
            interpreterStore.runFromPosition(cursorPosition);
        }
    };

    return (
        <div className="h-10 min-h-10 border-t border-zinc-800 bg-zinc-900 text-zinc-400">
            <div className="flex items-center px-2 h-full gap-1">
                {/* Run modes group */}
                <div className="flex items-center gap-1 pr-2 border-r border-zinc-700">
                    {isPaused ? (
                        <IconButton
                            icon={PlayIcon}
                            label="Resume"
                            onClick={() => interpreterStore.resume()}
                            variant="success"
                        />
                    ) : (
                        <>
                            <IconButton
                                icon={BoltIcon}
                                label="Run Really Fast (No delay, rare UI updates, no breakpoints)"
                                onClick={() => interpreterStore.runTurbo()}
                                disabled={isRunning}
                                variant="success"
                            />

                            <IconButton
                                icon={PlayIcon}
                                label="Run Smoothly (UI updates, breakpoints respected, slowest)"
                                onClick={() => interpreterStore.runSmooth()}
                                disabled={isRunning}
                                variant="success"
                            />

                            {/* Run with custom delay */}
                            <div className="flex items-center">
                                <IconButton
                                    icon={ClockIcon}
                                    label={`Run with ${delay}ms delay`}
                                    onClick={() => {
                                        interpreterStore.run(delay);
                                        setShowDelayInput(false);
                                    }}
                                    disabled={isRunning}
                                    variant="success"
                                />
                                {showDelayInput ? (
                                    <input
                                        type="number"
                                        value={delay}
                                        onChange={handleDelayChange}
                                        onBlur={() => setShowDelayInput(false)}
                                        onKeyDown={(e) => {
                                            if (e.key === 'Enter') {
                                                interpreterStore.run(delay);
                                                setShowDelayInput(false);
                                            }
                                            if (e.key === 'Escape') {
                                                setShowDelayInput(false);
                                            }
                                        }}
                                        className="ml-1 w-16 px-1 py-0.5 text-xs bg-zinc-800 border border-zinc-700 rounded text-zinc-300 focus:outline-none focus:border-zinc-600"
                                        placeholder="ms"
                                        min="0"
                                        max="1000"
                                        autoFocus
                                    />
                                ) : (
                                    <button
                                        onClick={() => setShowDelayInput(true)}
                                        className="ml-1 px-1 py-0.5 text-xs bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded text-zinc-400"
                                        disabled={isRunning}
                                    >
                                        {delay}ms
                                    </button>
                                )}
                            </div>
                        </>
                    )}
                </div>

                {/* Control buttons */}
                {isRunning && !isPaused && (
                    <IconButton
                        icon={PauseIcon}
                        label="Pause"
                        onClick={() => interpreterStore.pause()}
                        variant="warning"
                    />
                )}

                <IconButton
                    icon={StopIcon}
                    label="Stop"
                    onClick={() => interpreterStore.stop()}
                    disabled={!isRunning}
                    variant="danger"
                />

                <IconButton
                    icon={ForwardIcon}
                    label="Step"
                    onClick={() => interpreterStore.step()}
                    disabled={isRunning && !isPaused}
                    variant="info"
                />

                <IconButton
                    icon={CursorArrowRaysIcon}
                    label="Run from cursor"
                    onClick={handleRunFromCursor}
                    disabled={isRunning}
                    variant="success"
                />

                <div className="w-px h-6 bg-zinc-700 mx-1" />

                <IconButton
                    icon={XMarkIcon}
                    label="Clear Breakpoints"
                    onClick={() => interpreterStore.clearBreakpoints()}
                    variant="warning"
                />

                <IconButton
                    icon={ArrowPathIcon}
                    label="Reset"
                    onClick={() => interpreterStore.reset()}
                    variant="warning"
                />

                {/* Status indicator */}
                <div className="ml-auto flex items-center gap-2 text-xs">
                    {isRunning && (
                        <div className="flex items-center gap-1">
                            <div className={`w-2 h-2 rounded-full ${
                                isPaused
                                    ? 'bg-yellow-500'
                                    : 'bg-green-500 animate-pulse'
                            }`} />
                            <span className={
                                isPaused
                                    ? 'text-yellow-500'
                                    : 'text-green-500'
                            }>
                                {isPaused ? 'Paused' : 'Running'}
                            </span>
                        </div>
                    )}
                    {
                        isStopped && (
                            <div className="flex items-center gap-1">
                                <div className="w-2 h-2 rounded-full bg-red-500" />
                                <span className="text-red-500">Finished</span>
                            </div>
                        )
                    }
                </div>
            </div>
        </div>
    );
}