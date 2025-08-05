import { useState, useEffect } from "react";
import clsx from "clsx";
import { interpreterStore } from "../debugger/interpreter-facade.store.ts";
import { useStoreSubscribe } from "../../hooks/use-store-subscribe.tsx";
import { CameraIcon, TrashIcon, ArrowDownTrayIcon } from "@heroicons/react/24/outline";
import type {TapeSnapshot} from "../debugger/interpreter.store.ts";
import { Tooltip } from "../ui/tooltip.tsx";

const STORAGE_KEY = "brainfuck-tape-snapshots";

function loadSnapshots(): TapeSnapshot[] {
    try {
        const stored = localStorage.getItem(STORAGE_KEY);
        return stored ? JSON.parse(stored) : [];
    } catch (error) {
        console.error("Failed to load snapshots:", error);
        return [];
    }
}

function saveSnapshots(snapshots: TapeSnapshot[]) {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshots));
    } catch (error) {
        console.error("Failed to save snapshots:", error);
    }
}

export function Snapshots() {
    const [snapshots, setSnapshots] = useState<TapeSnapshot[]>(() => loadSnapshots());
    const [snapshotName, setSnapshotName] = useState("");
    
    const state = useStoreSubscribe(interpreterStore.state);
    const cellSize = useStoreSubscribe(interpreterStore.cellSize);
    const tapeSize = useStoreSubscribe(interpreterStore.tapeSize);

    // Save snapshots to localStorage whenever they change
    useEffect(() => {
        saveSnapshots(snapshots);
    }, [snapshots]);

    const createSnapshot = () => {
        if (!state) return;
        
        const name = snapshotName.trim() || `Snapshot ${snapshots.length + 1}`;
        
        // Convert tape to regular array for JSON serialization
        const tapeArray = Array.from(state.tape);
        
        const snapshot: TapeSnapshot = {
            id: Date.now().toString(),
            name,
            timestamp: Date.now(),
            tape: tapeArray,
            pointer: state.pointer,
            cellSize,
            tapeSize
        };
        
        setSnapshots([snapshot, ...snapshots]);
        setSnapshotName("");
    };

    const loadSnapshot = (snapshot: TapeSnapshot) => {
        interpreterStore.loadSnapshot(snapshot);
    };

    const deleteSnapshot = (id: string) => {
        setSnapshots(snapshots.filter(s => s.id !== id));
    };

    const formatDate = (timestamp: number) => {
        const date = new Date(timestamp);
        return date.toLocaleString();
    };

    return (
        <div className="h-full overflow-y-auto w-[268px] border-l border-zinc-800">
            {/* Header */}
            <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-4 py-3 z-10">
                <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">Tape Snapshots</h2>
            </div>

            {/* Content */}
            <div className="p-4 space-y-4">
                {/* Save new snapshot */}
                <div className="space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        Save Current State
                    </h3>
                    <div className="space-y-2">
                        <input
                            type="text"
                            value={snapshotName}
                            onChange={(e) => setSnapshotName(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    createSnapshot();
                                }
                            }}
                            placeholder="Snapshot name (optional)"
                            className="w-full px-2 py-1.5 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 
                                     focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        />
                        <button
                            onClick={createSnapshot}
                            disabled={!state}
                            className={clsx(
                                "w-full flex items-center justify-center gap-2 px-2 py-1.5 text-sm font-medium rounded transition-all",
                                state
                                    ? "bg-blue-600 hover:bg-blue-700 text-white"
                                    : "bg-zinc-800 text-zinc-600 cursor-not-allowed"
                            )}
                        >
                            <CameraIcon className="h-4 w-4" />
                            Save Snapshot
                        </button>
                    </div>
                </div>

                {/* Saved snapshots */}
                <div className="space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        Saved Snapshots ({snapshots.length})
                    </h3>
                    
                    {snapshots.length === 0 ? (
                        <p className="text-sm text-zinc-500 text-center py-4">
                            No snapshots saved yet
                        </p>
                    ) : (
                        <div className="space-y-1">
                            {snapshots.map((snapshot) => (
                                <div
                                    key={snapshot.id}
                                    className="group flex items-center gap-3 px-2 py-2 hover:bg-zinc-800/50 rounded transition-colors cursor-pointer"
                                    onClick={() => loadSnapshot(snapshot)}
                                >
                                    <div className="flex-1 min-w-0">
                                        <div className="flex items-baseline gap-2">
                                            <span className="text-sm text-zinc-200 truncate">
                                                {snapshot.name}
                                            </span>
                                            <span className="text-xs text-zinc-500">
                                                @{snapshot.pointer}
                                            </span>
                                        </div>
                                        <div className="text-xs text-zinc-500">
                                            {new Date(snapshot.timestamp).toLocaleDateString()} • {new Date(snapshot.timestamp).toLocaleTimeString()}
                                        </div>
                                    </div>
                                    
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            deleteSnapshot(snapshot.id);
                                        }}
                                        className="p-1 text-zinc-400 hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100"
                                    >
                                        <TrashIcon className="h-3.5 w-3.5" />
                                    </button>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}