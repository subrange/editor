import { useEffect, useRef, useState } from "react";
import { Modal, ModalFooter } from "../../common/modal.tsx";
import clsx from "clsx";

export interface MacroUsage {
    line: number;
    column: number;
    text: string;
    lineNumber: string;
}

interface MacroUsagesModalProps {
    macroName: string;
    usages: MacroUsage[];
    isOpen: boolean;
    onClose: () => void;
    onNavigate: (usage: MacroUsage) => void;
}

export function MacroUsagesModal({ 
    macroName, 
    usages, 
    isOpen, 
    onClose, 
    onNavigate
}: MacroUsagesModalProps) {
    const listRef = useRef<HTMLDivElement>(null);
    const [selectedIndex, setSelectedIndex] = useState(0);

    // Reset selected index when usages change
    useEffect(() => {
        setSelectedIndex(0);
    }, [usages]);

    // Scroll selected item into view
    useEffect(() => {
        if (listRef.current && selectedIndex >= 0 && selectedIndex < usages.length) {
            const selectedElement = listRef.current.children[selectedIndex] as HTMLElement;
            if (selectedElement) {
                selectedElement.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
            }
        }
    }, [selectedIndex, usages.length]);

    // Handle keyboard navigation
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            // Stop propagation for all handled keys to prevent editor from receiving them
            switch (e.key) {
                case "Enter":
                    e.preventDefault();
                    e.stopPropagation();
                    if (usages[selectedIndex]) {
                        onNavigate(usages[selectedIndex]);
                        onClose();
                    }
                    break;
                case "ArrowDown":
                    e.preventDefault();
                    e.stopPropagation();
                    setSelectedIndex(prev => 
                        prev < usages.length - 1 ? prev + 1 : prev
                    );
                    break;
                case "ArrowUp":
                    e.preventDefault();
                    e.stopPropagation();
                    setSelectedIndex(prev => prev > 0 ? prev - 1 : prev);
                    break;
                case "Tab":
                    e.preventDefault();
                    e.stopPropagation();
                    if (e.shiftKey) {
                        setSelectedIndex(prev => prev > 0 ? prev - 1 : prev);
                    } else {
                        setSelectedIndex(prev => 
                            prev < usages.length - 1 ? prev + 1 : prev
                        );
                    }
                    break;
            }
        };

        document.addEventListener("keydown", handleKeyDown, true);
        return () => document.removeEventListener("keydown", handleKeyDown, true);
    }, [isOpen, selectedIndex, usages, onNavigate, onClose]);

    const handleItemClick = (usage: MacroUsage, index: number) => {
        setSelectedIndex(index);
        onNavigate(usage);
        onClose();
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            position="top"
            width="w-[600px]"
            maxHeight="max-h-[400px]"
        >
            {/* Header - matching QuickNav style */}
            <div className="p-3 border-b border-zinc-700">
                <div className="text-sm text-zinc-400">
                    Usages of <span className="text-blue-400">@{macroName}</span>
                    <span className="ml-2 text-xs text-zinc-600">({usages.length} found)</span>
                </div>
            </div>

            {/* Results List */}
            <div 
                ref={listRef}
                className="overflow-auto flex-1 max-h-[320px]"
            >
                {usages.length === 0 ? (
                    <div className="p-4 text-center text-zinc-500 text-sm">
                        No usages found
                    </div>
                ) : (
                    usages.map((usage, index) => (
                        <div
                            key={`${usage.line}-${usage.column}`}
                            className={clsx(
                                "px-3 py-2 cursor-pointer flex items-center gap-2 text-sm",
                                index === selectedIndex
                                    ? "bg-zinc-800 text-zinc-100"
                                    : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
                            )}
                            onClick={() => handleItemClick(usage, index)}
                            onMouseEnter={() => setSelectedIndex(index)}
                        >
                            <span className="text-blue-400">@</span>
                            <span className="flex-1 truncate font-mono">
                                {usage.text}
                            </span>
                            <span className="text-xs text-zinc-600">
                                line {usage.lineNumber}
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