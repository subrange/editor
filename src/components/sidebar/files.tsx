import { useState, useEffect } from "react";
import { editorManager } from "../../services/editor-manager.service.ts";
import { DocumentIcon, TrashIcon, ArrowDownTrayIcon, ArrowDownIcon } from "@heroicons/react/24/outline";
import JSZip from "jszip";
import { Tooltip } from "../ui/tooltip";

const STORAGE_KEY = "brainfuck-saved-files";

export interface SavedFile {
    id: string;
    name: string;
    timestamp: number;
    mainContent: string;
    macroContent: string;
}

function loadFiles(): SavedFile[] {
    try {
        const stored = localStorage.getItem(STORAGE_KEY);
        return stored ? JSON.parse(stored) : [];
    } catch (error) {
        console.error("Failed to load files:", error);
        return [];
    }
}

function saveFiles(files: SavedFile[]) {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(files));
    } catch (error) {
        console.error("Failed to save files:", error);
    }
}

export function Files() {
    const [files, setFiles] = useState<SavedFile[]>(() => loadFiles());
    const [fileName, setFileName] = useState("");

    // Save files to localStorage whenever they change
    useEffect(() => {
        saveFiles(files);
    }, [files]);

    const saveCurrentFile = () => {
        const mainEditor = editorManager.getEditor("main");
        const macroEditor = editorManager.getEditor("macro");
        
        if (!mainEditor || !macroEditor) return;
        
        const name = fileName.trim() || `File ${files.length + 1}`;
        
        const file: SavedFile = {
            id: Date.now().toString(),
            name,
            timestamp: Date.now(),
            mainContent: mainEditor.getText(),
            macroContent: macroEditor.getText()
        };
        
        setFiles([file, ...files]);
        setFileName("");
    };

    const loadFile = (file: SavedFile) => {
        const mainEditor = editorManager.getEditor("main");
        const macroEditor = editorManager.getEditor("macro");
        
        if (mainEditor) {
            mainEditor.setContent(file.mainContent);
        }
        
        if (macroEditor) {
            macroEditor.setContent(file.macroContent);
        }
    };

    const deleteFile = (id: string) => {
        setFiles(files.filter(f => f.id !== id));
    };

    const formatDate = (timestamp: number) => {
        const date = new Date(timestamp);
        return date.toLocaleString();
    };

    const downloadFile = async (file: SavedFile) => {
        const hasMacroContent = file.macroContent.trim().length > 0;
        
        if (!hasMacroContent) {
            // Download single .bf file
            const blob = new Blob([file.mainContent], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `${file.name}.bf`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } else {
            // Create a zip archive with both files
            const zip = new JSZip();
            zip.file(`${file.name}.bf`, file.mainContent);
            zip.file(`${file.name}.bfm`, file.macroContent);
            
            try {
                const blob = await zip.generateAsync({ type: 'blob' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `${file.name}.zip`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            } catch (error) {
                console.error('Failed to create zip archive:', error);
            }
        }
    };

    return (
        <div className="h-full overflow-y-auto w-[268px] border-l border-zinc-800">
            {/* Header */}
            <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 z-10">
                <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">Files</h2>
            </div>

            {/* Content */}
            <div className="p-6 space-y-6">
                {/* Save new file */}
                <div className="space-y-3">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        Save Current Editors
                    </h3>
                    <div className="space-y-2">
                        <input
                            type="text"
                            value={fileName}
                            onChange={(e) => setFileName(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    saveCurrentFile();
                                }
                            }}
                            placeholder="File name (optional)"
                            className="w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 
                                     focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        />
                        <button
                            onClick={saveCurrentFile}
                            className="w-full flex items-center justify-center gap-2 px-3 py-2 text-sm font-medium rounded transition-all
                                     bg-blue-600 hover:bg-blue-700 text-white"
                        >
                            <DocumentIcon className="h-4 w-4" />
                            Save File
                        </button>
                    </div>
                </div>

                {/* Saved files */}
                <div className="space-y-3">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        Saved Files ({files.length})
                    </h3>
                    
                    {files.length === 0 ? (
                        <p className="text-sm text-zinc-500 text-center py-8">
                            No files saved yet
                        </p>
                    ) : (
                        <div className="space-y-2">
                            {files.map((file) => (
                                <div
                                    key={file.id}
                                    className="bg-zinc-800 rounded-lg border border-zinc-700 p-3 space-y-2"
                                >
                                    <div className="flex items-start justify-between">
                                        <div className="flex-1 min-w-0">
                                            <h4 className="text-sm font-medium text-zinc-200 truncate">
                                                {file.name}
                                            </h4>
                                            <p className="text-xs text-zinc-500 mt-1">
                                                {formatDate(file.timestamp)}
                                            </p>
                                        </div>
                                    </div>
                                    
                                    <div className="text-xs text-zinc-400 space-y-1">
                                        <div>Main: {file.mainContent.split('\n').length} lines</div>
                                        <div>Macro: {file.macroContent.split('\n').length} lines</div>
                                    </div>
                                    
                                    <div className="flex items-center gap-2 pt-1">
                                        <button
                                            onClick={() => loadFile(file)}
                                            className="flex-1 flex items-center justify-center gap-1 px-2 py-1 text-xs 
                                                     bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded transition-colors"
                                        >
                                            <ArrowDownTrayIcon className="h-3 w-3" />
                                            Load
                                        </button>
                                        <Tooltip 
                                            content={file.macroContent.trim() ? "Download as .zip archive" : "Download .bf file"}
                                            side="bottom"
                                        >
                                            <button
                                                onClick={() => downloadFile(file)}
                                                className="flex-1 flex items-center justify-center gap-1 px-2 py-1 text-xs 
                                                         bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded transition-colors"
                                            >
                                                <ArrowDownIcon className="h-3 w-3" />
                                                Download
                                            </button>
                                        </Tooltip>
                                        <button
                                            onClick={() => deleteFile(file.id)}
                                            className="p-1 text-zinc-500 hover:text-red-400 transition-colors"
                                            title="Delete file"
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