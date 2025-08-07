import { useEffect, useRef } from "react";
import { useStoreSubscribe } from "../../../hooks/use-store-subscribe.tsx";
import { AssemblyQuickNavStore, type AssemblyNavigationItem } from "../stores/assembly-quick-nav.store.ts";
import { Modal, ModalFooter } from "../../common/modal.tsx";
import clsx from "clsx";

interface AssemblyQuickNavProps {
    quickNavStore: AssemblyQuickNavStore;
    onNavigate: (item: AssemblyNavigationItem) => void;
    onHide?: () => void;
}

export function AssemblyQuickNav({ quickNavStore, onNavigate, onHide }: AssemblyQuickNavProps) {
    const state = useStoreSubscribe(quickNavStore.state);
    const inputRef = useRef<HTMLInputElement>(null);
    const listRef = useRef<HTMLDivElement>(null);

    // Focus input when visible
    useEffect(() => {
        if (state.isVisible && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    }, [state.isVisible]);

    // Scroll selected item into view
    useEffect(() => {
        if (listRef.current && state.selectedIndex >= 0) {
            const selectedElement = listRef.current.children[state.selectedIndex] as HTMLElement;
            if (selectedElement) {
                selectedElement.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
            }
        }
    }, [state.selectedIndex]);

    // Handle keyboard events
    useEffect(() => {
        if (!state.isVisible) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            switch (e.key) {
                case "Enter":
                    e.preventDefault();
                    e.stopPropagation();
                    const selectedItem = quickNavStore.getSelectedItem();
                    if (selectedItem) {
                        quickNavStore.hide();
                        // Small delay to ensure modal is hidden before navigation
                        setTimeout(() => {
                            onNavigate(selectedItem);
                        }, 0);
                    }
                    break;
                case "ArrowDown":
                    e.preventDefault();
                    quickNavStore.selectNext();
                    break;
                case "ArrowUp":
                    e.preventDefault();
                    quickNavStore.selectPrevious();
                    break;
                case "Tab":
                    e.preventDefault();
                    if (e.shiftKey) {
                        quickNavStore.selectPrevious();
                    } else {
                        quickNavStore.selectNext();
                    }
                    break;
            }
        };

        document.addEventListener("keydown", handleKeyDown, true);
        return () => document.removeEventListener("keydown", handleKeyDown, true);
    }, [state.isVisible, quickNavStore, onNavigate]);

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        quickNavStore.setQuery(e.target.value);
    };

    const handleItemClick = (item: AssemblyNavigationItem) => {
        quickNavStore.hide();
        // Small delay to ensure modal is hidden before navigation
        setTimeout(() => {
            onNavigate(item);
        }, 0);
    };

    const renderIcon = (type: 'label' | 'mark') => {
        if (type === 'label') {
            return <span className="text-rose-400">L</span>;
        } else {
            return <span className="text-yellow-400">#</span>;
        }
    };

    return (
        <Modal
            isOpen={state.isVisible}
            onClose={() => {
                quickNavStore.hide();
                onHide?.();
            }}
            position="top"
            width="w-[600px]"
            maxHeight="max-h-[400px]"
        >
            {/* Search Input */}
            <div className="p-3 border-b border-zinc-700">
                <input
                    ref={inputRef}
                    type="text"
                    value={state.query}
                    onChange={handleInputChange}
                    placeholder="Search for labels or marks..."
                    className="w-full bg-zinc-800 text-zinc-100 px-3 py-2 rounded text-sm outline-none focus:ring-1 focus:ring-blue-500"
                    onMouseDown={(e) => e.stopPropagation()}
                />
            </div>

            {/* Results List */}
            <div 
                ref={listRef}
                className="overflow-auto flex-1 max-h-[320px]"
            >
                {state.filteredItems.length === 0 ? (
                    <div className="p-4 text-center text-zinc-500 text-sm">
                        {state.query ? 'No matches found' : 'Type to search...'}
                    </div>
                ) : (
                    state.filteredItems.map((item, index) => (
                        <div
                            key={`${item.type}-${item.line}-${item.column}`}
                            className={clsx(
                                "px-3 py-2 cursor-pointer flex items-center gap-2 text-sm",
                                index === state.selectedIndex
                                    ? "bg-zinc-800 text-zinc-100"
                                    : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
                            )}
                            onClick={() => handleItemClick(item)}
                        >
                            {renderIcon(item.type)}
                            <span className="flex-1 truncate">{item.name}</span>
                            <span className="text-xs text-zinc-600">
                                line {item.line + 1}
                            </span>
                        </div>
                    ))
                )}
            </div>

            <ModalFooter>
                <div className="flex items-center gap-4">
                    <span>↑↓ Navigate</span>
                    <span>Enter Select</span>
                    <span>Esc Cancel</span>
                </div>
            </ModalFooter>
        </Modal>
    );
}