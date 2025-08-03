import { useState, useEffect } from "react";
import { editorManager } from "../../services/editor-manager.service.ts";
import { HashtagIcon } from "@heroicons/react/24/outline";

interface Mark {
    line: number;
    text: string;
    content: string;
}

export function Marks() {
    const [marks, setMarks] = useState<Mark[]>([]);

    useEffect(() => {
        const extractMarks = () => {
            const macroEditor = editorManager.getEditor("macro");
            if (!macroEditor) {
                setMarks([]);
                return;
            }

            const text = macroEditor.getText();
            const lines = text.split('\n');
            const extractedMarks: Mark[] = [];

            lines.forEach((line, index) => {
                const markMatch = line.match(/\/\/\s*MARK:\s*(.+)/);
                if (markMatch) {
                    extractedMarks.push({
                        line: index,
                        text: markMatch[1].trim(),
                        content: line
                    });
                }
            });

            setMarks(extractedMarks);
        };

        // Initial extraction
        extractMarks();

        // Subscribe to macro editor changes
        const macroEditor = editorManager.getEditor("macro");
        if (macroEditor) {
            const subscription = macroEditor.editorState.subscribe(() => {
                extractMarks();
            });

            return () => {
                subscription.unsubscribe();
            };
        }
    }, []);

    const navigateToMark = (mark: Mark) => {
        const macroEditor = editorManager.getEditor("macro");
        if (!macroEditor) return;

        // Set cursor position to the mark line
        macroEditor.setCursorPosition({
            line: mark.line,
            column: 0
        });

        // Ensure macro editor is active
        editorManager.setActiveEditor("macro");
    };

    return (
        <div className="h-full overflow-y-auto w-[268px] border-l border-zinc-800">
            {/* Header */}
            <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 z-10">
                <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">Marks</h2>
            </div>

            {/* Content */}
            <div className="p-4 space-y-4">
                <div className="space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        Macro Editor Marks ({marks.length})
                    </h3>

                    {marks.length === 0 ? (
                        <p className="text-sm text-zinc-500 text-center py-8">
                            No MARK comments found in macro editor
                        </p>
                    ) : (
                        <div className="space-y-0.5">
                            {marks.map((mark, index) => (
                                <button
                                    key={`${mark.line}-${index}`}
                                    onClick={() => navigateToMark(mark)}
                                    className="w-full text-left px-2 py-1 rounded text-xs
                                             hover:bg-zinc-800 transition-all group flex items-center gap-2"
                                >
                                    <span className="text-zinc-600 group-hover:text-zinc-500 font-mono text-[10px] min-w-[3ch] text-right">
                                        {mark.line + 1}
                                    </span>
                                    <span className="text-zinc-300 group-hover:text-zinc-100 truncate">
                                        {mark.text}
                                    </span>
                                </button>
                            ))}
                        </div>
                    )}
                </div>

                <div className="text-xs text-zinc-500 space-y-1">
                    <p>Use // MARK: comments in the macro editor to create navigation points.</p>
                    <p>Click on a mark to jump to its location.</p>
                </div>
            </div>
        </div>
    );
}