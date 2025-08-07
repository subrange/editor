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
    assemblyContent?: string;
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
        const assemblyEditor = editorManager.getEditor("assembly");
        
        if (!mainEditor || !macroEditor) return;
        
        const name = fileName.trim() || `File ${files.length + 1}`;
        
        const file: SavedFile = {
            id: Date.now().toString(),
            name,
            timestamp: Date.now(),
            mainContent: mainEditor.getText(),
            macroContent: macroEditor.getText(),
            assemblyContent: assemblyEditor?.getText() || ''
        };
        
        setFiles([file, ...files]);
        setFileName("");
    };

    const loadFile = (file: SavedFile) => {
        const mainEditor = editorManager.getEditor("main");
        const macroEditor = editorManager.getEditor("macro");
        const assemblyEditor = editorManager.getEditor("assembly");
        
        if (mainEditor) {
            mainEditor.setContent(file.mainContent);
        }
        
        if (macroEditor) {
            macroEditor.setContent(file.macroContent);
        }
        
        if (assemblyEditor && file.assemblyContent) {
            assemblyEditor.setContent(file.assemblyContent);
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
        const hasAssemblyContent = file.assemblyContent?.trim().length > 0;
        
        if (!hasMacroContent && !hasAssemblyContent) {
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
            // Create a zip archive with all files
            const zip = new JSZip();
            zip.file(`${file.name}.bf`, file.mainContent);
            if (hasMacroContent) {
                zip.file(`${file.name}.bfm`, file.macroContent);
            }
            if (hasAssemblyContent) {
                zip.file(`${file.name}.asm`, file.assemblyContent!);
            }
            
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
            <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-4 py-3 z-10">
                <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">Files</h2>
            </div>

            {/* Content */}
            <div className="p-4 space-y-4">
                {/* Save new file */}
                <div className="space-y-2">
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
                            className="w-full px-2 py-1.5 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 
                                     focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        />
                        <button
                            onClick={saveCurrentFile}
                            className="w-full flex items-center justify-center gap-2 px-2 py-1.5 text-sm font-medium rounded transition-all
                                     bg-blue-600 hover:bg-blue-700 text-white"
                        >
                            <DocumentIcon className="h-4 w-4" />
                            Save File
                        </button>
                    </div>
                </div>

                {/* Saved files */}
                <div className="space-y-2">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        Saved Files ({files.length})
                    </h3>
                    
                    {files.length === 0 ? (
                        <p className="text-sm text-zinc-500 text-center py-4">
                            No files saved yet
                        </p>
                    ) : (
                        <div className="space-y-1">
                            {files.map((file) => {
                                const mainLines = file.mainContent.split('\n').length;
                                const macroLines = file.macroContent.split('\n').length;
                                const hasMacros = file.macroContent.trim().length > 0;
                                
                                return (
                                    <div
                                        key={file.id}
                                        className="group flex items-center gap-3 px-2 py-2 hover:bg-zinc-800/50 rounded transition-colors cursor-pointer"
                                        onClick={() => loadFile(file)}
                                    >
                                        <div className="flex-1 min-w-0">
                                            <div className="flex items-baseline gap-2">
                                                <span className="text-sm text-zinc-200 truncate">
                                                    {file.name}
                                                </span>
                                                {hasMacros && (
                                                    <span className="text-xs text-blue-400">M</span>
                                                )}
                                                {file.assemblyContent?.trim() && (
                                                    <span className="text-xs text-green-400">A</span>
                                                )}
                                            </div>
                                            <div className="text-xs text-zinc-500">
                                                {new Date(file.timestamp).toLocaleDateString()} • {macroLines}L
                                            </div>
                                        </div>
                                        
                                        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                            <Tooltip content={hasMacros ? "Download .zip" : "Download .bf"} side="left">
                                                <button
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        downloadFile(file);
                                                    }}
                                                    className="p-1 text-zinc-400 hover:text-zinc-200 transition-colors"
                                                >
                                                    <ArrowDownIcon className="h-3.5 w-3.5" />
                                                </button>
                                            </Tooltip>
                                            <button
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    deleteFile(file.id);
                                                }}
                                                className="p-1 text-zinc-400 hover:text-red-400 transition-colors"
                                            >
                                                <TrashIcon className="h-3.5 w-3.5" />
                                            </button>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}