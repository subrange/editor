import { useState, useEffect } from "react";
import clsx from "clsx";
import { interpreterStore } from "../debugger/interpreter-facade.store.ts";
import { useStoreSubscribe } from "../../hooks/use-store-subscribe.tsx";
import { CameraIcon, TrashIcon, ArrowDownTrayIcon } from "@heroicons/react/24/outline";
import type {TapeSnapshot} from "../debugger/interpreter.store.ts";
import { Tooltip } from "../ui/tooltip.tsx";
import { tapeLabelsStore } from "../../stores/tape-labels.store";

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
        const serialized = JSON.stringify(snapshots);
        localStorage.setItem(STORAGE_KEY, serialized);
    } catch (error) {
        console.error("Failed to save snapshots:", error);
        if (error instanceof DOMException && error.name === 'QuotaExceededError') {
            alert("Failed to save snapshots: Storage quota exceeded. Please delete some old snapshots to free up space.");
        } else {
            alert(`Failed to save snapshots: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }
}

export function Snapshots() {
    const [snapshots, setSnapshots] = useState<TapeSnapshot[]>(() => loadSnapshots());
    const [snapshotName, setSnapshotName] = useState("");
    
    const state = useStoreSubscribe(interpreterStore.state);
    const cellSize = useStoreSubscribe(interpreterStore.cellSize);
    const tapeSize = useStoreSubscribe(interpreterStore.tapeSize);
    const labels = useStoreSubscribe(tapeLabelsStore.labels);

    // Save snapshots to localStorage whenever they change
    useEffect(() => {
        saveSnapshots(snapshots);
    }, [snapshots]);

    const createSnapshot = () => {
        if (!state) return;
        
        const name = snapshotName.trim() || `Snapshot ${snapshots.length + 1}`;
        
        try {
            // Find the highest non-zero index to minimize storage
            let maxIndex = 0;
            for (let i = state.tape.length - 1; i >= 0; i--) {
                if (state.tape[i] !== 0) {
                    maxIndex = i;
                    break;
                }
            }
            
            // Convert only the used portion of tape to regular array
            const usedLength = Math.max(maxIndex + 1, state.pointer + 1);
            const tapeArray = Array.from(state.tape.slice(0, usedLength));
            
            const snapshot: TapeSnapshot = {
                id: Date.now().toString(),
                name,
                timestamp: Date.now(),
                tape: tapeArray,
                pointer: state.pointer,
                cellSize,
                tapeSize,
                labels: {
                    lanes: { ...labels.lanes },
                    columns: { ...labels.columns },
                    cells: { ...labels.cells }
                }
            };
            
            // Test if we can serialize it
            const serialized = JSON.stringify(snapshot);
            
            // Check if it fits in localStorage (usually ~5-10MB limit)
            if (serialized.length > 5 * 1024 * 1024) { // 5MB safety limit
                alert(`Snapshot is too large (${(serialized.length / 1024 / 1024).toFixed(2)}MB). The tape has ${usedLength.toLocaleString()} non-empty cells. Consider using a smaller tape or clearing unused cells.`);
                return;
            }
            
            setSnapshots([snapshot, ...snapshots]);
            setSnapshotName("");
        } catch (error) {
            console.error("Failed to create snapshot:", error);
            if (error instanceof RangeError) {
                alert(`Cannot create snapshot: The tape is too large (${state.tape.length.toLocaleString()} cells). The browser cannot allocate enough memory for this operation.`);
            } else {
                alert(`Failed to create snapshot: ${error instanceof Error ? error.message : 'Unknown error'}`);
            }
        }
    };

    const loadSnapshot = (snapshot: TapeSnapshot) => {
        interpreterStore.loadSnapshot(snapshot);
        
        // Always clear existing labels first
        tapeLabelsStore.clearAllLabels();
        
        // Restore labels if available
        if (snapshot.labels) {
            // Restore lane labels
            Object.entries(snapshot.labels.lanes).forEach(([index, label]) => {
                tapeLabelsStore.setLaneLabel(parseInt(index), label);
            });
            
            // Restore column labels
            Object.entries(snapshot.labels.columns).forEach(([index, label]) => {
                tapeLabelsStore.setColumnLabel(parseInt(index), label);
            });
            
            // Restore cell labels
            if (snapshot.labels.cells) {
                Object.entries(snapshot.labels.cells).forEach(([index, label]) => {
                    tapeLabelsStore.setCellLabel(parseInt(index), label);
                });
            }
        }
    };

    const deleteSnapshot = (id: string) => {
        setSnapshots(snapshots.filter(s => s.id !== id));
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