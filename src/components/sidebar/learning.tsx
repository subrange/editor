import { useState } from 'react';
import { learningStore, type LearningCategory, type LearningItem } from '../../stores/learning.store';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe';
import { ChevronRightIcon, ChevronDownIcon } from '@heroicons/react/20/solid';
import { editorManager } from '../../services/editor-manager.service';
import { useLocalStorageState } from '../../hooks/use-local-storage-state';
import clsx from 'clsx';
import { interpreterStore } from '../debugger/interpreter-facade.store';
import { settingsStore } from '../../stores/settings.store';
import { tapeLabelsStore } from '../../stores/tape-labels.store';

export function Learning() {
    const learningState = useStoreSubscribe(learningStore.state);
    const [expandedCategories, setExpandedCategories] = useLocalStorageState<string[]>('learning-expanded-categories', []);
    const [expandedSubcategories, setExpandedSubcategories] = useLocalStorageState<string[]>('learning-expanded-subcategories', []);
    const [selectedItemId, setSelectedItemId] = useState<string | null>(null);

    const toggleCategory = (categoryId: string) => {
        setExpandedCategories((prev: string[]) =>
            prev.includes(categoryId)
                ? prev.filter((id: string) => id !== categoryId)
                : [...prev, categoryId]
        );
    };

    const toggleSubcategory = (subcategoryId: string) => {
        setExpandedSubcategories((prev: string[]) =>
            prev.includes(subcategoryId)
                ? prev.filter((id: string) => id !== subcategoryId)
                : [...prev, subcategoryId]
        );
    };

    const loadItem = (item: LearningItem) => {
        // Update selected item
        setSelectedItemId(item.id);
        learningStore.selectItem(item);

        // Configure editors based on item config
        const { editorConfig, content, interpreterConfig, debuggerConfig, labels } = item;

        // Show/hide editors as needed
        const showMainEditor = localStorage.getItem('showMainEditor');
        const showMacroEditor = localStorage.getItem('showMacroEditor');
        
        if (editorConfig.showMainEditor !== undefined && showMainEditor !== String(editorConfig.showMainEditor)) {
            localStorage.setItem('showMainEditor', String(editorConfig.showMainEditor));
        }
        
        if (editorConfig.showMacroEditor !== undefined && showMacroEditor !== String(editorConfig.showMacroEditor)) {
            localStorage.setItem('showMacroEditor', String(editorConfig.showMacroEditor));
        }

        // Set main editor mode if specified
        if (editorConfig.mainEditorMode) {
            localStorage.setItem('mainEditorMode', JSON.stringify(editorConfig.mainEditorMode));
        }

        // Apply interpreter configuration
        if (interpreterConfig) {
            if (interpreterConfig.tapeSize !== undefined) {
                interpreterStore.setTapeSize(interpreterConfig.tapeSize);
            }
            if (interpreterConfig.cellSize !== undefined) {
                interpreterStore.setCellSize(interpreterConfig.cellSize);
            }
        }

        // Apply debugger configuration
        if (debuggerConfig) {
            if (debuggerConfig.viewMode !== undefined) {
                settingsStore.setDebuggerViewMode(debuggerConfig.viewMode);
            }
            if (debuggerConfig.laneCount !== undefined) {
                interpreterStore.setLaneCount(debuggerConfig.laneCount);
            }
        }

        // Clear existing labels first, then apply new ones
        if (labels) {
            // Clear all labels first
            tapeLabelsStore.clearAllLabels();
            
            // Apply lane labels
            if (labels.lanes) {
                Object.entries(labels.lanes).forEach(([index, label]) => {
                    tapeLabelsStore.setLaneLabel(Number(index), label);
                });
            }
            
            // Apply column labels
            if (labels.columns) {
                Object.entries(labels.columns).forEach(([index, label]) => {
                    tapeLabelsStore.setColumnLabel(Number(index), label);
                });
            }
            
            // Apply cell labels
            if (labels.cells) {
                Object.entries(labels.cells).forEach(([index, label]) => {
                    tapeLabelsStore.setCellLabel(Number(index), label);
                });
            }
        } else {
            // Clear all labels if no labels are specified
            tapeLabelsStore.clearAllLabels();
        }

        // Ensure debugger is visible
        localStorage.setItem('debugCollapsed', 'false');

        // Load content into editors
        if (content.mainEditor !== undefined) {
            const mainEditor = editorManager.getEditor('main');
            if (mainEditor) {
                mainEditor.setContent(content.mainEditor);
            }
        }

        if (content.macroEditor !== undefined) {
            const macroEditor = editorManager.getEditor('macro');
            if (macroEditor) {
                macroEditor.setContent(content.macroEditor);
            }
        }

        if (content.assemblyEditor !== undefined) {
            const assemblyEditor = editorManager.getEditor('assembly');
            if (assemblyEditor) {
                assemblyEditor.setContent(content.assemblyEditor);
            }
        }

        // Force a reload to apply editor visibility changes
        window.location.reload();
    };

    return (
        <div className="h-full overflow-y-auto w-[268px] border-l border-zinc-800">
            {/* Header */}
            <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-4 py-3 z-10">
                <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">Learning</h2>
            </div>

            {/* Content */}
            <div className="p-4 space-y-2">
                {learningState.categories.map((category: LearningCategory) => (
                    <div key={category.id} className="space-y-1">
                        {/* Category Header */}
                        <button
                            onClick={() => toggleCategory(category.id)}
                            className="w-full flex items-center justify-between p-2 hover:bg-zinc-800/50 rounded transition-colors group"
                        >
                            <div className="flex items-center gap-2">
                                {expandedCategories.includes(category.id) ? (
                                    <ChevronDownIcon className="w-4 h-4 text-zinc-500" />
                                ) : (
                                    <ChevronRightIcon className="w-4 h-4 text-zinc-500" />
                                )}
                                <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                                    {category.icon} {category.name}
                                </span>
                            </div>
                        </button>

                        {/* Subcategories */}
                        {expandedCategories.includes(category.id) && (
                            <div className="ml-3 space-y-1">
                                {category.subcategories.map(subcategory => (
                                    <div key={subcategory.id}>
                                        {/* Subcategory Header */}
                                        <button
                                            onClick={() => toggleSubcategory(subcategory.id)}
                                            className="w-full flex items-center justify-between p-1.5 hover:bg-zinc-800/30 rounded transition-colors group"
                                        >
                                            <div className="flex items-center gap-2">
                                                {expandedSubcategories.includes(subcategory.id) ? (
                                                    <ChevronDownIcon className="w-3 h-3 text-zinc-600" />
                                                ) : (
                                                    <ChevronRightIcon className="w-3 h-3 text-zinc-600" />
                                                )}
                                                <span className="text-xs font-medium text-zinc-400 group-hover:text-zinc-300">
                                                    {subcategory.name}
                                                </span>
                                            </div>
                                        </button>

                                        {/* Items */}
                                        {expandedSubcategories.includes(subcategory.id) && (
                                            <div className="mt-1 space-y-1">
                                                {subcategory.items.map(item => (
                                                    <button
                                                        key={item.id}
                                                        onClick={() => loadItem(item)}
                                                        className={clsx(
                                                            "w-full text-left p-2 rounded transition-colors",
                                                            selectedItemId === item.id
                                                                ? "bg-blue-500/20 border border-blue-500/50"
                                                                : "bg-zinc-800 hover:bg-zinc-700"
                                                        )}
                                                    >
                                                        <p className="text-sm text-zinc-200 font-medium">
                                                            {item.name}
                                                        </p>
                                                        <p className="text-xs text-zinc-500 mt-0.5">
                                                            {item.description}
                                                        </p>
                                                    </button>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                ))}

                {/* Info message */}
                <div className="mt-6 p-3 bg-zinc-800/50 rounded border border-zinc-700">
                    <p className="text-xs text-zinc-400">
                        💡 Select a tutorial to load example code into the editors. The IDE will configure the appropriate editors for each lesson.
                    </p>
                </div>

                {/* Links Section */}
                <div className="mt-6 space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Helpful Links</h3>
                    
                    <div className="space-y-1">
                        <a
                            href="https://esolangs.org/wiki/Brainfuck"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="block p-2 bg-zinc-800/50 hover:bg-zinc-700/50 rounded text-xs text-zinc-300 hover:text-zinc-100 transition-colors"
                        >
                            📖 Brainfuck Language Reference
                        </a>

                        <a 
                            href="https://esolangs.org/wiki/Brainfuck_algorithms"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="block p-2 bg-zinc-800/50 hover:bg-zinc-700/50 rounded text-xs text-zinc-300 hover:text-zinc-100 transition-colors"
                        >
                            📚 Brainfuck Algorithms Wiki
                        </a>
                        
                        <a 
                            href="http://www.hevanet.com/cristofd/brainfuck/"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="block p-2 bg-zinc-800/50 hover:bg-zinc-700/50 rounded text-xs text-zinc-300 hover:text-zinc-100 transition-colors"
                        >
                            🧑‍💻 Daniel Cristofani's BF Page
                        </a>
                        
                        <a 
                            href="https://www.reddit.com/r/brainfuck/"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="block p-2 bg-zinc-800/50 hover:bg-zinc-700/50 rounded text-xs text-zinc-300 hover:text-zinc-100 transition-colors"
                        >
                            💬 Reddit Brainfuck Community
                        </a>
                        
                        <a
                            href="https://copy.sh/brainfuck/"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="block p-2 bg-zinc-800/50 hover:bg-zinc-700/50 rounded text-xs text-zinc-300 hover:text-zinc-100 transition-colors"
                        >
                            🎮 Copy.sh Brainfuck Interpreter
                        </a>

                    </div>
                </div>
            </div>
        </div>
    );
}