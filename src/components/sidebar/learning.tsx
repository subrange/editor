import { useState, useEffect } from 'react';
import { learningStore, type LearningCategory, type LearningItem } from '../../stores/learning.store';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe';
import { ChevronRightIcon } from '@heroicons/react/20/solid';
import { BookOpenIcon } from '@heroicons/react/24/outline';
import { editorManager } from '../../services/editor-manager.service';
import { useLocalStorageState } from '../../hooks/use-local-storage-state';
import clsx from 'clsx';
import { interpreterStore } from '../debugger/interpreter-facade.store';
import { settingsStore } from '../../stores/settings.store';
import { tapeLabelsStore } from '../../stores/tape-labels.store';
import { MarkdownViewer } from '../markdown-viewer';

export function Learning() {
    const learningState = useStoreSubscribe(learningStore.state);
    const [expandedCategories, setExpandedCategories] = useLocalStorageState<string[]>('learning-expanded-categories', []);
    const [expandedSubcategories, setExpandedSubcategories] = useLocalStorageState<string[]>('learning-expanded-subcategories', []);
    const [selectedItemId, setSelectedItemId] = useLocalStorageState<string | null>('learning-selected-item', null);
    const [showTutorial, setShowTutorial] = useState(false);
    const [showAssemblyTutorial, setShowAssemblyTutorial] = useState(false);
    const [showIDEOverview, setShowIDEOverview] = useState(false);
    const [showEditorGuide, setShowEditorGuide] = useState(false);
    const [showDebuggerGuide, setShowDebuggerGuide] = useState(false);
    const [showSettingsGuide, setShowSettingsGuide] = useState(false);

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

    // Auto-restore last selected item on mount
    useEffect(() => {
        if (selectedItemId && !learningStore.state.value.selectedItem) {
            // Find the item by ID across all categories
            for (const category of learningState.categories) {
                for (const subcategory of category.subcategories) {
                    const foundItem = subcategory.items.find(item => item.id === selectedItemId);
                    if (foundItem) {
                        learningStore.selectItem(foundItem);
                        return;
                    }
                }
            }
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []); // Run once on mount - selectedItemId and learningState are intentionally not in deps

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
            <div className="p-3 space-y-1">
                {/* Interactive Examples Section */}
                <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-2">Interactive Examples</h3>
                
                {learningState.categories.map((category: LearningCategory) => (
                    <div key={category.id} className="">
                        {/* Category Header */}
                        <button
                            onClick={() => toggleCategory(category.id)}
                            className="w-full flex items-center gap-1.5 px-2 py-1.5 hover:bg-zinc-800/40 rounded-md transition-all duration-150 group"
                        >
                            <div className="flex items-center gap-1.5 flex-1">
                                <div className="transition-transform duration-200" style={{
                                    transform: expandedCategories.includes(category.id) ? 'rotate(90deg)' : 'rotate(0deg)'
                                }}>
                                    <ChevronRightIcon className="w-3.5 h-3.5 text-zinc-500 group-hover:text-zinc-400" />
                                </div>
                                <span className="text-[13px] font-medium text-zinc-300 group-hover:text-zinc-100 select-none">
                                    {category.icon} {category.name}
                                </span>
                            </div>
                            {expandedCategories.includes(category.id) && (
                                <span className="text-[10px] text-zinc-600 font-mono">
                                    {category.subcategories.length}
                                </span>
                            )}
                        </button>

                        {/* Subcategories */}
                        {expandedCategories.includes(category.id) && (
                            <div className="ml-3 border-l border-zinc-800 mt-1">
                                {category.subcategories.map((subcategory) => (
                                    <div key={subcategory.id} className="relative">
                                        {/* Tree connector line */}
                                        <div className="absolute left-0 top-3 w-3 h-px bg-zinc-800" />
                                        
                                        {/* Subcategory Header */}
                                        <button
                                            onClick={() => toggleSubcategory(subcategory.id)}
                                            className="w-full flex items-center gap-1.5 ml-3 px-2 py-1 hover:bg-zinc-800/30 rounded-md transition-all duration-150 group"
                                        >
                                            <div className="flex items-center gap-1.5 flex-1">
                                                <div className="transition-transform duration-200" style={{
                                                    transform: expandedSubcategories.includes(subcategory.id) ? 'rotate(90deg)' : 'rotate(0deg)'
                                                }}>
                                                    <ChevronRightIcon className="w-3 h-3 text-zinc-600 group-hover:text-zinc-500" />
                                                </div>
                                                <span className="text-[12px] font-medium text-zinc-400 group-hover:text-zinc-200 select-none">
                                                    {subcategory.name}
                                                </span>
                                            </div>
                                            {expandedSubcategories.includes(subcategory.id) && (
                                                <span className="text-[10px] text-zinc-700 font-mono">
                                                    {subcategory.items.length}
                                                </span>
                                            )}
                                        </button>

                                        {/* Items */}
                                        {expandedSubcategories.includes(subcategory.id) && (
                                            <div className="ml-6 border-l border-zinc-800/50 mt-0.5">
                                                {subcategory.items.map((item) => (
                                                    <div key={item.id} className="relative">
                                                        {/* Tree connector for items */}
                                                        <div className="absolute left-0 top-4 w-3 h-px bg-zinc-800/50" />
                                                        
                                                        <button
                                                            onClick={() => loadItem(item)}
                                                            className={clsx(
                                                                "w-full text-left ml-3 p-2 rounded-md transition-all duration-150",
                                                                selectedItemId === item.id
                                                                    ? "bg-gradient-to-r from-blue-500/20 to-purple-500/20 border border-blue-500/40 shadow-sm shadow-blue-500/10"
                                                                    : "hover:bg-zinc-800/40 border border-transparent hover:border-zinc-700/50"
                                                            )}
                                                        >
                                                            <div className="flex items-start gap-2">
                                                                {/* Item indicator */}
                                                                <div className={clsx(
                                                                    "w-1.5 h-1.5 rounded-full mt-1.5 flex-shrink-0 transition-colors",
                                                                    selectedItemId === item.id
                                                                        ? "bg-blue-400 shadow-sm shadow-blue-400/50"
                                                                        : "bg-zinc-600"
                                                                )} />
                                                                <div className="flex-1 min-w-0">
                                                                    <p className={clsx(
                                                                        "text-[12px] font-medium leading-snug",
                                                                        selectedItemId === item.id
                                                                            ? "text-zinc-100"
                                                                            : "text-zinc-300"
                                                                    )}>
                                                                        {item.name}
                                                                    </p>
                                                                    <p className="text-[11px] text-zinc-500 mt-0.5 leading-snug line-clamp-2">
                                                                        {item.description}
                                                                    </p>
                                                                </div>
                                                            </div>
                                                        </button>
                                                    </div>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                ))}

                {/* Links Section */}
                <div className="mt-6 space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">IDE Documentation</h3>
                    
                    <div className="space-y-1">
                        <button
                            onClick={() => setShowIDEOverview(true)}
                            className="w-full flex items-center gap-2 p-2 bg-gradient-to-r from-indigo-600/20 to-violet-600/20 hover:from-indigo-600/30 hover:to-violet-600/30 border border-indigo-500/30 rounded text-xs text-zinc-100 transition-all"
                        >
                            <BookOpenIcon className="w-4 h-4 text-indigo-400" />
                            <span className="font-medium">Complete IDE Features Overview</span>
                        </button>
                        
                        <button
                            onClick={() => setShowEditorGuide(true)}
                            className="w-full flex items-center gap-2 p-2 bg-gradient-to-r from-pink-600/20 to-rose-600/20 hover:from-pink-600/30 hover:to-rose-600/30 border border-pink-500/30 rounded text-xs text-zinc-100 transition-all"
                        >
                            <BookOpenIcon className="w-4 h-4 text-pink-400" />
                            <span className="font-medium">Editor System Guide</span>
                        </button>
                        
                        <button
                            onClick={() => setShowDebuggerGuide(true)}
                            className="w-full flex items-center gap-2 p-2 bg-gradient-to-r from-orange-600/20 to-amber-600/20 hover:from-orange-600/30 hover:to-amber-600/30 border border-orange-500/30 rounded text-xs text-zinc-100 transition-all"
                        >
                            <BookOpenIcon className="w-4 h-4 text-orange-400" />
                            <span className="font-medium">Debugger & Execution Guide</span>
                        </button>
                        
                        <button
                            onClick={() => setShowSettingsGuide(true)}
                            className="w-full flex items-center gap-2 p-2 bg-gradient-to-r from-cyan-600/20 to-teal-600/20 hover:from-cyan-600/30 hover:to-teal-600/30 border border-cyan-500/30 rounded text-xs text-zinc-100 transition-all"
                        >
                            <BookOpenIcon className="w-4 h-4 text-cyan-400" />
                            <span className="font-medium">Settings & Configuration Guide</span>
                        </button>
                    </div>
                </div>

                {/* Language Documentation Section */}
                <div className="mt-6 space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">Language Documentation</h3>
                    
                    <div className="space-y-1">
                        <button
                            onClick={() => setShowTutorial(true)}
                            className="w-full flex items-center gap-2 p-2 bg-gradient-to-r from-blue-600/20 to-purple-600/20 hover:from-blue-600/30 hover:to-purple-600/30 border border-blue-500/30 rounded text-xs text-zinc-100 transition-all"
                        >
                            <BookOpenIcon className="w-4 h-4 text-blue-400" />
                            <span className="font-medium">Brainfuck Macro Language Tutorial</span>
                        </button>
                        
                        <button
                            onClick={() => setShowAssemblyTutorial(true)}
                            className="w-full flex items-center gap-2 p-2 bg-gradient-to-r from-green-600/20 to-emerald-600/20 hover:from-green-600/30 hover:to-emerald-600/30 border border-green-500/30 rounded text-xs text-zinc-100 transition-all"
                        >
                            <BookOpenIcon className="w-4 h-4 text-green-400" />
                            <span className="font-medium">Ripple VM Assembly & Architecture</span>
                        </button>
                    </div>
                </div>

                {/* External Links Section */}
                <div className="mt-6 space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">External Resources</h3>
                    
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
            
            {/* Markdown Tutorial Viewer */}
            {showTutorial && (
                <MarkdownViewer 
                    filePath="/BRAINFUCK_MACRO_TUTORIAL.md"
                    onClose={() => setShowTutorial(false)}
                />
            )}
            
            {/* Assembly Tutorial Viewer */}
            {showAssemblyTutorial && (
                <MarkdownViewer 
                    filePath="/RIPPLE_ASSEMBLY_TUTORIAL.md"
                    onClose={() => setShowAssemblyTutorial(false)}
                />
            )}
            
            {/* IDE Overview Documentation */}
            {showIDEOverview && (
                <MarkdownViewer 
                    filePath="/IDE_FEATURES_OVERVIEW.md"
                    onClose={() => setShowIDEOverview(false)}
                />
            )}
            
            {/* Editor Guide Documentation */}
            {showEditorGuide && (
                <MarkdownViewer 
                    filePath="/IDE_EDITOR_GUIDE.md"
                    onClose={() => setShowEditorGuide(false)}
                />
            )}
            
            {/* Debugger Guide Documentation */}
            {showDebuggerGuide && (
                <MarkdownViewer 
                    filePath="/IDE_DEBUGGER_GUIDE.md"
                    onClose={() => setShowDebuggerGuide(false)}
                />
            )}
            
            {/* Settings Guide Documentation */}
            {showSettingsGuide && (
                <MarkdownViewer 
                    filePath="/IDE_SETTINGS_GUIDE.md"
                    onClose={() => setShowSettingsGuide(false)}
                />
            )}
        </div>
    );
}