import { useState } from 'react';
import { learningStore, type LearningCategory, type LearningItem } from '../../stores/learning.store';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe';
import { ChevronRightIcon, ChevronDownIcon } from '@heroicons/react/20/solid';
import { editorManager } from '../../services/editor-manager.service';
import { useLocalStorageState } from '../../hooks/use-local-storage-state';
import clsx from 'clsx';

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
        const { editorConfig, content } = item;

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
            </div>
        </div>
    );
}