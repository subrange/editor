import { useState, useEffect } from "react";
import clsx from "clsx";
import { interpreterStore } from "../debugger/interpreter-facade.store.ts";
import { useStoreSubscribe } from "../../hooks/use-store-subscribe.tsx";
import { CameraIcon, TrashIcon, ArrowDownTrayIcon } from "@heroicons/react/24/outline";

interface TapeSnapshot {
    id: string;
    name: string;
    timestamp: number;
    tape: number[];
    pointer: number;
    cellSize: number;
    tapeSize: number;
}

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
        // First set the tape and cell sizes
        if (snapshot.tapeSize !== tapeSize) {
            interpreterStore.setTapeSize(snapshot.tapeSize);
        }
        if (snapshot.cellSize !== cellSize) {
            interpreterStore.setCellSize(snapshot.cellSize);
        }
        
        // Get current state to modify
        const currentState = interpreterStore.state.getValue();
        
        // Create new tape array of correct type
        let newTape: Uint8Array | Uint16Array | Uint32Array;
        if (snapshot.cellSize === 256) {
            newTape = new Uint8Array(snapshot.tapeSize);
        } else if (snapshot.cellSize === 65536) {
            newTape = new Uint16Array(snapshot.tapeSize);
        } else {
            newTape = new Uint32Array(snapshot.tapeSize);
        }
        
        // Copy snapshot data
        for (let i = 0; i < Math.min(snapshot.tape.length, newTape.length); i++) {
            newTape[i] = snapshot.tape[i];
        }
        
        // Update interpreter state
        interpreterStore.state.next({
            ...currentState,
            tape: newTape,
            pointer: snapshot.pointer
        });
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
            <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 z-10">
                <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">Tape Snapshots</h2>
            </div>

            {/* Content */}
            <div className="p-6 space-y-6">
                {/* Save new snapshot */}
                <div className="space-y-3">
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
                            className="w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 
                                     focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        />
                        <button
                            onClick={createSnapshot}
                            disabled={!state}
                            className={clsx(
                                "w-full flex items-center justify-center gap-2 px-3 py-2 text-sm font-medium rounded transition-all",
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
                <div className="space-y-3">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        Saved Snapshots ({snapshots.length})
                    </h3>
                    
                    {snapshots.length === 0 ? (
                        <p className="text-sm text-zinc-500 text-center py-8">
                            No snapshots saved yet
                        </p>
                    ) : (
                        <div className="space-y-2">
                            {snapshots.map((snapshot) => (
                                <div
                                    key={snapshot.id}
                                    className="bg-zinc-800 rounded-lg border border-zinc-700 p-3 space-y-2"
                                >
                                    <div className="flex items-start justify-between">
                                        <div className="flex-1 min-w-0">
                                            <h4 className="text-sm font-medium text-zinc-200 truncate">
                                                {snapshot.name}
                                            </h4>
                                            <p className="text-xs text-zinc-500 mt-1">
                                                {formatDate(snapshot.timestamp)}
                                            </p>
                                        </div>
                                    </div>
                                    
                                    <div className="text-xs text-zinc-400 space-y-1">
                                        <div>Pointer: {snapshot.pointer}</div>
                                        <div>Cell size: {snapshot.cellSize === 256 ? '8-bit' : snapshot.cellSize === 65536 ? '16-bit' : '32-bit'}</div>
                                        <div>Tape size: {snapshot.tapeSize} cells</div>
                                    </div>
                                    
                                    <div className="flex items-center gap-2 pt-1">
                                        <button
                                            onClick={() => loadSnapshot(snapshot)}
                                            className="flex-1 flex items-center justify-center gap-1 px-2 py-1 text-xs 
                                                     bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded transition-colors"
                                        >
                                            <ArrowDownTrayIcon className="h-3 w-3" />
                                            Load
                                        </button>
                                        <button
                                            onClick={() => deleteSnapshot(snapshot.id)}
                                            className="p-1 text-zinc-500 hover:text-red-400 transition-colors"
                                            title="Delete snapshot"
                                        >
                                            <TrashIcon className="h-4 w-4" />
                                        </button>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}