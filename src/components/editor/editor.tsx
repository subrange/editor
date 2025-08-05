import {VSep} from "../helper-components.tsx";
import {useStoreSubscribe} from "../../hooks/use-store-subscribe.tsx";
import {EditorStore, type Line} from "./stores/editor.store.ts";
import clsx from "clsx";
import {type AppCommand, keybindingsService, type KeybindingState} from "../../services/keybindings.service.ts";
import {useRef, useState, useEffect, useCallback} from "react";
import {ProgressiveMacroTokenizer} from "./services/macro-tokenizer-progressive.ts";
import {type MacroExpansionError} from "../../services/macro-expander/macro-expander.ts";
import {SearchBar} from "./components/search-bar.tsx";
import {QuickNav} from "./components/quick-nav.tsx";
import {type NavigationItem} from "./stores/quick-nav.store.ts";
import {LineNumbersPanel} from "./components/line-numbers-panel.tsx";
import {LinesPanel} from "./components/lines-panel.tsx";
import {Minimap} from "./components/minimap.tsx";


export interface EditorProps {
    store: EditorStore;
    onFocus?: () => void;
    onBlur?: () => void;
}

export function Editor({store, onFocus, onBlur}: EditorProps) {
    const editorRef = useRef<HTMLDivElement>(null);
    const focused = useStoreSubscribe(store.focused);
    const showMinimap = useStoreSubscribe(store.showMinimap);
    const [editorContainerWidth, setEditorContainerWidth] = useState(0);
    const [editorScrollLeft, setEditorScrollLeft] = useState(0);
    const [macroErrors, setMacroErrors] = useState<MacroExpansionError[]>([]);
    
    // Extract navigation items from content
    const extractNavigationItems = useCallback((lines: Line[]): NavigationItem[] => {
        const items: NavigationItem[] = [];
        const tokenizer = store.getTokenizer();
        
        // Get macro definitions if using ProgressiveMacroTokenizer
        if (tokenizer instanceof ProgressiveMacroTokenizer) {
            const macros = tokenizer.state?.macroDefinitions || [];
            macros.forEach(macro => {
                if (macro.sourceLocation) {
                    items.push({
                        type: 'macro',
                        name: macro.name,
                        line: macro.sourceLocation.line,
                        column: macro.sourceLocation.column,
                    });
                }
            });
        }
        
        // Extract MARK comments
        lines.forEach((line, index) => {
            const markMatch = line.text.match(/\/\/\s*MARK:\s*(.+)/);
            if (markMatch) {
                items.push({
                    type: 'mark',
                    name: markMatch[1].trim(),
                    line: index,
                    column: 0,
                });
            }
        });
        
        // Sort by line number
        return items.sort((a, b) => a.line - b.line);
    }, [store]);

    // Re-run search when editor content changes
    useEffect(() => {
        let debounceTimer: number;
        let lastContent = "";
        
        const subscription = store.editorState.subscribe((state) => {
            const searchState = store.searchStore.state.value;
            if (searchState.query && searchState.isVisible) {
                // Only re-search if content actually changed (not just cursor position)
                const currentContent = state.lines.map(l => l.text).join('\n');
                if (currentContent !== lastContent) {
                    lastContent = currentContent;
                    // Debounce the search to avoid running it on every keystroke
                    clearTimeout(debounceTimer);
                    debounceTimer = setTimeout(() => {
                        store.performSearch(searchState.query);
                        // Don't jump to matches when content changes during editing
                    }, 100);
                }
            }
            
            // Update navigation items
            const navItems = extractNavigationItems(state.lines);
            store.quickNavStore.setItems(navItems);
        });
        
        return () => {
            clearTimeout(debounceTimer);
            subscription.unsubscribe();
        };
    }, [store, extractNavigationItems]);

    // Update navigation items when tokenizer state changes
    useEffect(() => {
        const tokenizer = store.getTokenizer();
        if (tokenizer instanceof ProgressiveMacroTokenizer) {
            const unsubscribe = tokenizer.onStateChange(() => {
                // Update navigation items when macros change
                const navItems = extractNavigationItems(store.editorState.value.lines);
                store.quickNavStore.setItems(navItems);
                
                // Update errors
                const errors = tokenizer.state?.expanderErrors || [];
                setMacroErrors(errors);
            });
            return unsubscribe;
        }
    }, [store, extractNavigationItems]);

    // Track editor container width
    useEffect(() => {
        if (!editorRef.current) return;

        const resizeObserver = new ResizeObserver((entries) => {
            requestAnimationFrame(() => {
                for (const entry of entries) {
                    // Get the width of the editor minus the line numbers panel (64px) and separator (1px)
                    // Also subtract minimap width (120px) if visible
                    const minimapWidth = showMinimap ? 120 : 0;
                    const width = entry.contentRect.width - 65 - minimapWidth;
                    setEditorContainerWidth(width);
                }
            });
        });

        resizeObserver.observe(editorRef.current);
        // Initial width calculation
        const minimapWidth = showMinimap ? 120 : 0;
        setEditorContainerWidth(editorRef.current.offsetWidth - 65 - minimapWidth);

        return () => resizeObserver.disconnect();
    }, [showMinimap]);

    const handleEditorScroll = (e: React.UIEvent<HTMLDivElement>) => {
        setEditorScrollLeft((e.target as HTMLDivElement).scrollLeft);
    };

    function addEditorKeybindings() {
        // Use editor-specific keybinding state to avoid conflicts
        const keybindingState = `editor_${store.getId()}` as KeybindingState;
        keybindingsService.pushKeybindings(keybindingState, [
            keybindingsService.createKeybinding("arrowright", "editor.moveright" as AppCommand),
            keybindingsService.createKeybinding("arrowleft", "editor.moveleft" as AppCommand),
            keybindingsService.createKeybinding("arrowup", "editor.moveup" as AppCommand),
            keybindingsService.createKeybinding("arrowdown", "editor.movedown" as AppCommand),

            keybindingsService.createKeybinding("meta+z", "editor.undo" as AppCommand),
            keybindingsService.createKeybinding("meta+y", "editor.redo" as AppCommand),
            keybindingsService.createKeybinding("meta+shift+z", "editor.redo" as AppCommand),

            // Add selection keybindings
            keybindingsService.createKeybinding("meta+a", "editor.selectall" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowright", "editor.selectright" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowright", "editor.selectwordright" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowleft", "editor.selectleft" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowleft", "editor.selectwordleft" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowup", "editor.selectup" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowup", "editor.selectlineup" as AppCommand),
            keybindingsService.createKeybinding("shift+arrowdown", "editor.selectdown" as AppCommand),
            keybindingsService.createKeybinding("shift+alt+arrowdown", "editor.selectlinedown" as AppCommand),

            // Line start/end movement
            keybindingsService.createKeybinding("meta+arrowleft", "editor.movelinestart" as AppCommand),
            keybindingsService.createKeybinding("meta+arrowright", "editor.movelineend" as AppCommand),
            keybindingsService.createKeybinding("meta+shift+arrowleft", "editor.selectlinestart" as AppCommand),
            keybindingsService.createKeybinding("meta+shift+arrowright", "editor.selectlineend" as AppCommand),

            // Copy/Cut/Paste
            keybindingsService.createKeybinding("meta+c", "editor.copy" as AppCommand),
            keybindingsService.createKeybinding("meta+x", "editor.cut" as AppCommand),
            keybindingsService.createKeybinding("meta+v", "editor.paste" as AppCommand),

            // Search
            keybindingsService.createKeybinding("meta+f", "editor.search" as AppCommand),
            
            // Quick Navigation
            keybindingsService.createKeybinding("meta+p", "editor.quicknav" as AppCommand),
        ])

        store.focus();
        onFocus?.();
    }

    function removeEditorKeybindings() {
        const keybindingState = `editor_${store.getId()}` as KeybindingState;
        keybindingsService.removeKeybindings(keybindingState);
        store.blur();
        onBlur?.();
    }

    return (
        <div className="flex overflow-hidden grow-1 relative">
            <SearchBar
                searchStore={store.searchStore}
                editorStore={store}
                onSearch={(query: string, jumpToFirst?: boolean) => {
                    store.performSearch(query);
                    if (jumpToFirst) {
                        // Only jump to first match when explicitly typing in search box
                        setTimeout(() => {
                            const match = store.searchStore.getCurrentMatch();
                            if (match) {
                                store.setCursorPosition({ 
                                    line: match.line, 
                                    column: match.startColumn 
                                });
                            }
                        }, 0);
                    }
                }}
                onHide={() => {
                    // Focus the editor when search bar is hidden
                    setTimeout(() => {
                        editorRef.current?.focus();
                    }, 0);
                }}
            />
            <QuickNav
                quickNavStore={store.quickNavStore}
                editorStore={store}
                onNavigate={(item: NavigationItem) => {
                    // Focus the editor first
                    editorRef.current?.focus();
                    // Then navigate after a small delay to ensure focus is established
                    setTimeout(() => {
                        // Set navigation flag before moving cursor
                        store.isNavigating.next(true);
                        store.setCursorPosition({
                            line: item.line,
                            column: item.column
                        });
                    }, 0);
                }}
                onHide={() => {
                    // Focus the editor when quick nav is hidden
                    setTimeout(() => {
                        editorRef.current?.focus();
                    }, 0);
                }}
            />
            {store.getId() === 'macro' && macroErrors.length > 0 && (
                <div 
                    className="absolute top-2 right-2 bg-red-900 text-red-200 px-3 py-1 rounded-md flex items-center gap-2 cursor-pointer z-50 hover:bg-red-800 shadow-lg"
                    onClick={() => {
                        // Jump to the first error with a location
                        const firstError = macroErrors.find(e => e.location);
                        if (firstError?.location) {
                            store.isNavigating.next(true);
                            store.setCursorPosition({
                                line: firstError.location.line,
                                column: firstError.location.column
                            });
                        }
                    }}
                >
                    <span className="text-sm font-medium">
                        {macroErrors.length} {macroErrors.length === 1 ? 'error' : 'errors'}
                    </span>
                    <span className="text-xs opacity-75">Click to jump</span>
                </div>
            )}
            <div
                ref={editorRef}
                className={clsx(
                    "flex grow-1 bg-zinc-950 font-mono text-sm inset-shadow-sm overflow-auto relative select-none outline-0",
                    {
                        "border border-zinc-700": focused,
                        "border border-transparent": !focused
                    }
                )}
                onFocus={addEditorKeybindings}
                tabIndex={0}
                onBlur={removeEditorKeybindings}
                onScroll={handleEditorScroll}
            >
                <div className="flex relative grow-1 overflow-visible min-h-0 h-fit">
                    <LineNumbersPanel store={store}/>
                    <VSep className="sticky left-16 z-1 top-0 bottom-0"></VSep>
                    <LinesPanel store={store} editorWidth={editorContainerWidth} scrollLeft={editorScrollLeft}/>
                    {showMinimap && <Minimap store={store} />}
                </div>
            </div>
        </div>
    )
}