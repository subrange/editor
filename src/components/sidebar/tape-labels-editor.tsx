import { useState } from 'react';
import { tapeLabelsStore } from '../../stores/tape-labels.store';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe';
import { PlusIcon, XMarkIcon } from '@heroicons/react/24/solid';

export function TapeLabelsEditor() {
    const labels = useStoreSubscribe(tapeLabelsStore.labels);
    const [showAddLane, setShowAddLane] = useState(false);
    const [showAddColumn, setShowAddColumn] = useState(false);
    const [newLaneIndex, setNewLaneIndex] = useState('');
    const [newLaneLabel, setNewLaneLabel] = useState('');
    const [newColumnIndex, setNewColumnIndex] = useState('');
    const [newColumnLabel, setNewColumnLabel] = useState('');
    const [showAddCell, setShowAddCell] = useState(false);
    const [newCellIndex, setNewCellIndex] = useState('');
    const [newCellLabel, setNewCellLabel] = useState('');

    const handleAddLaneLabel = () => {
        const index = parseInt(newLaneIndex);
        if (!isNaN(index) && newLaneLabel.trim()) {
            tapeLabelsStore.setLaneLabel(index, newLaneLabel.trim());
            setNewLaneIndex('');
            setNewLaneLabel('');
            setShowAddLane(false);
        }
    };

    const handleAddColumnLabel = () => {
        const index = parseInt(newColumnIndex);
        if (!isNaN(index) && newColumnLabel.trim()) {
            tapeLabelsStore.setColumnLabel(index, newColumnLabel.trim());
            setNewColumnIndex('');
            setNewColumnLabel('');
            setShowAddColumn(false);
        }
    };

    const handleAddCellLabel = () => {
        const index = parseInt(newCellIndex);
        if (!isNaN(index) && newCellLabel.trim()) {
            tapeLabelsStore.setCellLabel(index, newCellLabel.trim());
            setNewCellIndex('');
            setNewCellLabel('');
            setShowAddCell(false);
        }
    };

    return (
        <div className="space-y-4">
            <div>
                <div className="flex items-center justify-between mb-2">
                    <h3 className="text-sm font-medium text-zinc-300">Cell Labels</h3>
                    <button
                        onClick={() => setShowAddCell(!showAddCell)}
                        className="p-1 hover:bg-zinc-800 rounded transition-colors"
                    >
                        <PlusIcon className="w-4 h-4 text-zinc-400" />
                    </button>
                </div>

                {showAddCell && (
                    <div className="space-y-2 mb-2">
                        <div className="flex gap-2">
                            <input
                                type="number"
                                placeholder="Index"
                                value={newCellIndex}
                                onChange={(e) => setNewCellIndex(e.target.value)}
                                className="w-12 px-1.5 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded"
                            />
                            <input
                                type="text"
                                placeholder="Label"
                                value={newCellLabel}
                                onChange={(e) => setNewCellLabel(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && handleAddCellLabel()}
                                className="flex-1 px-2 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded"
                            />
                        </div>
                        <button
                            onClick={handleAddCellLabel}
                            className="w-full px-2 py-1 text-xs bg-zinc-700 hover:bg-zinc-600 rounded transition-colors"
                        >
                            Add Cell Label
                        </button>
                    </div>
                )}

                <div className="space-y-1 max-h-48 overflow-y-auto">
                    {Object.entries(labels.cells).map(([index, label]) => (
                        <div key={index} className="flex items-center gap-1 p-1 text-xs bg-zinc-800 rounded">
                            <span className="text-zinc-400 shrink-0">#{index}:</span>
                            <span className="text-zinc-300 truncate flex-1">{label}</span>
                            <button
                                onClick={() => tapeLabelsStore.removeCellLabel(parseInt(index))}
                                className="p-0.5 hover:bg-zinc-700 rounded transition-colors shrink-0"
                            >
                                <XMarkIcon className="w-3 h-3 text-zinc-500" />
                            </button>
                        </div>
                    ))}
                    {Object.keys(labels.cells).length === 0 && (
                        <p className="text-xs text-zinc-500">No cell labels defined</p>
                    )}
                </div>
            </div>

            <div>
                <div className="flex items-center justify-between mb-2">
                    <h3 className="text-sm font-medium text-zinc-300">Lane Labels</h3>
                    <button
                        onClick={() => setShowAddLane(!showAddLane)}
                        className="p-1 hover:bg-zinc-800 rounded transition-colors"
                    >
                        <PlusIcon className="w-4 h-4 text-zinc-400" />
                    </button>
                </div>
                
                {showAddLane && (
                    <div className="space-y-2 mb-2">
                        <div className="flex gap-2">
                            <input
                                type="number"
                                placeholder="Index"
                                value={newLaneIndex}
                                onChange={(e) => setNewLaneIndex(e.target.value)}
                                className="w-12 px-1.5 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded"
                            />
                            <input
                                type="text"
                                placeholder="Label"
                                value={newLaneLabel}
                                onChange={(e) => setNewLaneLabel(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && handleAddLaneLabel()}
                                className="flex-1 px-2 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded"
                            />
                        </div>
                        <button
                            onClick={handleAddLaneLabel}
                            className="w-full px-2 py-1 text-xs bg-zinc-700 hover:bg-zinc-600 rounded transition-colors"
                        >
                            Add Lane Label
                        </button>
                    </div>
                )}
                
                <div className="space-y-1">
                    {Object.entries(labels.lanes).map(([index, label]) => (
                        <div key={index} className="flex items-center gap-1 p-1 text-xs bg-zinc-800 rounded">
                            <span className="text-zinc-400 shrink-0">#{index}:</span>
                            <span className="text-zinc-300 truncate flex-1">{label}</span>
                            <button
                                onClick={() => tapeLabelsStore.removeLaneLabel(parseInt(index))}
                                className="p-0.5 hover:bg-zinc-700 rounded transition-colors shrink-0"
                            >
                                <XMarkIcon className="w-3 h-3 text-zinc-500" />
                            </button>
                        </div>
                    ))}
                    {Object.keys(labels.lanes).length === 0 && (
                        <p className="text-xs text-zinc-500">No lane labels defined</p>
                    )}
                </div>
            </div>

            <div>
                <div className="flex items-center justify-between mb-2">
                    <h3 className="text-sm font-medium text-zinc-300">Column Labels</h3>
                    <button
                        onClick={() => setShowAddColumn(!showAddColumn)}
                        className="p-1 hover:bg-zinc-800 rounded transition-colors"
                    >
                        <PlusIcon className="w-4 h-4 text-zinc-400" />
                    </button>
                </div>
                
                {showAddColumn && (
                    <div className="space-y-2 mb-2">
                        <div className="flex gap-2">
                            <input
                                type="number"
                                placeholder="Index"
                                value={newColumnIndex}
                                onChange={(e) => setNewColumnIndex(e.target.value)}
                                className="w-12 px-1.5 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded"
                            />
                            <input
                                type="text"
                                placeholder="Label"
                                value={newColumnLabel}
                                onChange={(e) => setNewColumnLabel(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && handleAddColumnLabel()}
                                className="flex-1 px-2 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded"
                            />
                        </div>
                        <button
                            onClick={handleAddColumnLabel}
                            className="w-full px-2 py-1 text-xs bg-zinc-700 hover:bg-zinc-600 rounded transition-colors"
                        >
                            Add Column Label
                        </button>
                    </div>
                )}
                
                <div className="space-y-1">
                    {Object.entries(labels.columns).map(([index, label]) => (
                        <div key={index} className="flex items-center gap-1 p-1 text-xs bg-zinc-800 rounded">
                            <span className="text-zinc-400 shrink-0">#{index}:</span>
                            <span className="text-zinc-300 truncate flex-1">{label}</span>
                            <button
                                onClick={() => tapeLabelsStore.removeColumnLabel(parseInt(index))}
                                className="p-0.5 hover:bg-zinc-700 rounded transition-colors shrink-0"
                            >
                                <XMarkIcon className="w-3 h-3 text-zinc-500" />
                            </button>
                        </div>
                    ))}
                    {Object.keys(labels.columns).length === 0 && (
                        <p className="text-xs text-zinc-500">No column labels defined</p>
                    )}
                </div>
            </div>


            {(Object.keys(labels.lanes).length > 0 || Object.keys(labels.columns).length > 0 || Object.keys(labels.cells).length > 0) && (
                <button
                    onClick={() => tapeLabelsStore.clearAllLabels()}
                    className="w-full px-3 py-1 text-xs text-zinc-400 bg-zinc-800 hover:bg-zinc-700 rounded transition-colors"
                >
                    Clear All Labels
                </button>
            )}
        </div>
    );
}