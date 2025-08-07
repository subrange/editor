import {BehaviorSubject, Subscription} from "rxjs";
import {keybindingsService} from "../../../services/keybindings.service.ts";
import { type ITokenizer } from "../../../services/editor-manager.service.ts";
import { SearchStore, type SearchMatch } from "./search.store.ts";
import { QuickNavStore } from "./quick-nav.store.ts";

export type Position = {
    line: number;
    column: number;
}

export type Selection = {
    anchor: Position;
    focus: Position;
}

export type Range = {
    start: Position;
    end: Position;
}

export type Line = {
    text: string;
}

type EditorState = {
    selection: Selection;
    lines: Line[];
    mode: "normal" | "insert" | "command";
}

type CommandData =
    | { type: "insert"; position: Position; text: string }
    | { type: "delete"; range: Range; deletedText?: string }
    | { type: "move"; from: Position; to: Position }
    | { type: "composite"; commands: CommandData[] };

class CommandExecutor {
    execute(command: CommandData, state: EditorState): EditorState {
        switch (command.type) {
            case "insert":
                return this.executeInsert(command, state);
            case "delete":
                return this.executeDelete(command, state);
            case "move":
                return this.executeMove(command, state);
            case "composite":
                return command.commands.reduce((s, cmd) => this.execute(cmd, s), state);
        }
    }

    undo(command: CommandData, state: EditorState): EditorState {
        switch (command.type) {
            case "insert":
                // Calculate the end position after the inserted text
                { const endPos = this.calculateEndPosition(command.position, command.text);
                return this.executeDelete({
                    range: {
                        start: command.position,
                        end: endPos
                    },
                    deletedText: command.text
                }, state); }

            case "delete":
                if (!command.deletedText) {
                    console.error("Cannot undo delete - no deleted text stored");
                    return state;
                }
                return this.executeInsert({
                    position: command.range.start,
                    text: command.deletedText
                }, state);

            case "move":
                return this.executeMove({
                    from: command.to,
                    to: command.from
                }, state);

            case "composite":
                // Undo in reverse order
                return [...command.commands].reverse().reduce((s, cmd) => this.undo(cmd, s), state);
        }
    }

    static extractText(range: Range, state: EditorState): string {
        const { start, end } = range;

        if (start.line === end.line) {
            // Single line
            return state.lines[start.line].text.slice(start.column, end.column);
        }

        // Multi-line
        let text = '';

        // First line: from start column to end
        text += state.lines[start.line].text.slice(start.column) + '\n';

        // Middle lines: entire lines
        for (let i = start.line + 1; i < end.line; i++) {
            text += state.lines[i].text + '\n';
        }

        // Last line: from beginning to end column
        text += state.lines[end.line].text.slice(0, end.column);

        return text;
    }

    private calculateEndPosition(start: Position, text: string): Position {
        const lines = text.split('\n');

        if (lines.length === 1) {
            return {
                line: start.line,
                column: start.column + text.length
            };
        }

        return {
            line: start.line + lines.length - 1,
            column: lines[lines.length - 1].length
        };
    }

    private executeInsert(command: { position: Position; text: string }, state: EditorState): EditorState {
        const newLines = [...state.lines];
        const line = newLines[command.position.line];

        if (!line) {
            console.warn(`Line ${command.position.line} does not exist.`);
            return state;
        }

        const before = line.text.slice(0, command.position.column);
        const after = line.text.slice(command.position.column);

        // Split the inserted text by newlines
        const insertedLines = command.text.split('\n');

        if (insertedLines.length === 1) {
            // Simple insertion without newlines
            newLines[command.position.line] = {
                text: before + command.text + after
            };

            const newSelection: Selection = {
                anchor: {
                    line: command.position.line,
                    column: command.position.column + command.text.length
                },
                focus: {
                    line: command.position.line,
                    column: command.position.column + command.text.length
                }
            };

            return { ...state, lines: newLines, selection: newSelection };
        }

        // Multi-line insertion
        const linesToInsert: Line[] = [];

        // First line: existing text before cursor + first line of inserted text
        linesToInsert.push({ text: before + insertedLines[0] });

        // Middle lines (if any)
        for (let i = 1; i < insertedLines.length - 1; i++) {
            linesToInsert.push({ text: insertedLines[i] });
        }

        // Last line: last line of inserted text + existing text after cursor
        linesToInsert.push({ text: insertedLines[insertedLines.length - 1] + after });

        // Replace the current line with all new lines
        newLines.splice(command.position.line, 1, ...linesToInsert);

        // Calculate new cursor position
        const newLine = command.position.line + insertedLines.length - 1;
        const newColumn = insertedLines[insertedLines.length - 1].length;

        const newSelection: Selection = {
            anchor: { line: newLine, column: newColumn },
            focus: { line: newLine, column: newColumn }
        };

        return {
            ...state,
            lines: newLines,
            selection: newSelection
        };
    }

    private executeDelete(command: { range: Range; deletedText?: string }, state: EditorState): EditorState {
        // Store the deleted text if not already stored
        if (!command.deletedText) {
            command.deletedText = CommandExecutor.extractText(command.range, state);
        }

        const newLines = [...state.lines.map(l => ({ ...l }))];
        const startLine = newLines[command.range.start.line];
        const endLine = newLines[command.range.end.line];

        if (!startLine || !endLine) {
            console.warn(`Invalid range: ${command.range.start.line}-${command.range.end.line}`);
            return state;
        }

        let newSelection: Selection;

        // Delete the text in the specified range
        if (command.range.start.line === command.range.end.line) {
            // Same line deletion
            const before = startLine.text.slice(0, command.range.start.column);
            const after = startLine.text.slice(command.range.end.column);

            // Check if we're deleting at the beginning of a line
            if (command.range.start.column === 0 && command.range.end.column === 0 && command.range.start.line > 0) {
                // Merge with previous line
                const prevLine = newLines[command.range.start.line - 1];
                const mergeColumn = prevLine.text.length;
                prevLine.text = prevLine.text + startLine.text;

                // Remove the current line
                newLines.splice(command.range.start.line, 1);

                // Update selection to end of previous line
                newSelection = {
                    anchor: { line: command.range.start.line - 1, column: mergeColumn },
                    focus: { line: command.range.start.line - 1, column: mergeColumn }
                };
            } else {
                // Normal same-line deletion
                startLine.text = before + after;
                newLines[command.range.start.line] = startLine;

                newSelection = {
                    anchor: command.range.start,
                    focus: command.range.start
                };
            }
        } else {
            // Multi-line deletion
            const startText = startLine.text.slice(0, command.range.start.column);
            const endText = endLine.text.slice(command.range.end.column);

            // Merge the start and end lines
            newLines[command.range.start.line] = {
                text: startText + endText
            };

            // Remove lines in between (including the end line)
            newLines.splice(command.range.start.line + 1, command.range.end.line - command.range.start.line);

            newSelection = {
                anchor: command.range.start,
                focus: command.range.start
            };
        }

        return {
            ...state,
            lines: newLines,
            selection: newSelection
        };
    }

    private executeMove(command: { from: Position; to: Position }, state: EditorState): EditorState {
        const newSelection: Selection = {
            anchor: command.to,
            focus: command.to
        };

        return {
            ...state,
            selection: newSelection
        };
    }
}

function positionsEqual(pos1: Position, pos2: Position): boolean {
    return pos1.line === pos2.line && pos1.column === pos2.column;
}

function selectionToRange(selection: Selection): Range {
    const { anchor, focus } = selection;

    // Normalize the range so start is always before end
    if (anchor.line < focus.line || (anchor.line === focus.line && anchor.column <= focus.column)) {
        return { start: anchor, end: focus };
    } else {
        return { start: focus, end: anchor };
    }
}

function isSelectionCollapsed(selection: Selection): boolean {
    return positionsEqual(selection.anchor, selection.focus);
}

class UndoRedo {
    private history: CommandData[] = [];
    private index: number = -1;
    private executor = new CommandExecutor();
    private lastCommandTime: number = 0;
    private readonly MERGE_THRESHOLD_MS = 1000;

    execute(command: CommandData, state: EditorState): EditorState {
        // Truncate history if needed
        if (this.index < this.history.length - 1) {
            this.history = this.history.slice(0, this.index + 1);
        }

        // Try to merge with last command
        const merged = this.tryMerge(this.history[this.index], command);
        if (merged) {
            this.history[this.index] = merged;
        } else {
            this.history.push(command);
            this.index++;
        }

        this.lastCommandTime = Date.now();
        return this.executor.execute(command, state);
    }

    undo(state: EditorState): EditorState {
        if (this.index < 0) return state;

        const command = this.history[this.index];
        const newState = this.executor.undo(command, state);
        this.index--;

        return newState;
    }

    redo(state: EditorState): EditorState {
        if (this.index >= this.history.length - 1) return state;

        this.index++;
        const command = this.history[this.index];
        return this.executor.execute(command, state);
    }

    clear() {
        this.history = [];
        this.index = -1;
    }

    private tryMerge(last: CommandData | undefined, current: CommandData): CommandData | null {
        if (!last) return null;

        // Don't merge if too much time has passed
        const timeDiff = Date.now() - this.lastCommandTime;
        if (timeDiff > this.MERGE_THRESHOLD_MS) {
            return null;
        }

        // Merge consecutive inserts at the same position
        if (last.type === "insert" && current.type === "insert") {
            const lastEnd = this.calculateEndPosition(last.position, last.text);
            if (positionsEqual(lastEnd, current.position)) {
                return {
                    type: "insert",
                    position: last.position,
                    text: last.text + current.text
                };
            }
        }

        // Merge consecutive deletes (backspace)
        if (last.type === "delete" && current.type === "delete") {
            // Check if current delete is right before the last delete
            if (positionsEqual(current.range.end, last.range.start)) {
                return {
                    type: "delete",
                    range: {
                        start: current.range.start,
                        end: last.range.end
                    },
                    deletedText: (current.deletedText || '') + (last.deletedText || '')
                };
            }
        }

        // Merge consecutive moves
        if (last.type === "move" && current.type === "move" &&
            positionsEqual(last.to, current.from)) {
            return {
                type: "move",
                from: last.from,
                to: current.to
            };
        }

        return null;
    }

    private calculateEndPosition(start: Position, text: string): Position {
        const lines = text.split('\n');

        if (lines.length === 1) {
            return {
                line: start.line,
                column: start.column + text.length
            };
        }

        return {
            line: start.line + lines.length - 1,
            column: lines[lines.length - 1].length
        };
    }
}

export interface IEditorSettings {
    showDebug?: boolean;
    showMinimap?: boolean;
}

export class EditorStore {
    private id: string;
    private tokenizer: ITokenizer;
    private subscriptions: Subscription[] = [];
    public searchStore: SearchStore;
    public quickNavStore: QuickNavStore;
    
    // Cursor position history for navigation
    private cursorHistory: Position[] = [];
    private cursorHistoryIndex: number = -1;
    private readonly MAX_HISTORY_SIZE = 50;
    
    // Konami code tracking
    private konamiSequence: string[] = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
    private konamiProgress: number = 0;
    private konamiTimer: number = 0;
    
    public editorState = new BehaviorSubject<EditorState>({
        selection: {
            anchor: {line: 0, column: 0},
            focus: {line: 0, column: 0}
        },
        lines: [
            {text: ""}
        ],
        mode: "insert"
    });

    public cursorBlinkState = new BehaviorSubject<boolean>(true);
    private cursorBlinkRestoreTimeout: number = 0;
    public isNavigating = new BehaviorSubject<boolean>(false);

    private undoRedo = new UndoRedo();

    public focused = new BehaviorSubject(false);

    public showDebug = false;
    public showMinimap = new BehaviorSubject<boolean>(false);

    constructor(
        id: string,
        tokenizer: ITokenizer,
        settings?: IEditorSettings,
        initialContent?: string,
        initialMode?: "normal" | "insert" | "command",
    ) {
        this.id = id;
        this.tokenizer = tokenizer;
        this.searchStore = new SearchStore();
        this.quickNavStore = new QuickNavStore();

        this.showDebug = settings?.showDebug || false;
        this.showMinimap.next(settings?.showMinimap || false);
        
        // Load from localStorage using editor-specific key
        const savedState = localStorage.getItem(`editorState_${id}`);
        if (savedState) {
            try {
                const parsedState = JSON.parse(savedState);
                this.editorState.next(parsedState);
            } catch (e) {
                console.error("Failed to parse saved editor state:", e);
            }
        }

        // Apply initial content if provided and no saved state
        if (initialContent && !savedState) {
            const lines = initialContent.split('\n').map(text => ({ text }));
            this.editorState.next({
                ...this.editorState.getValue(),
                lines: lines.length > 0 ? lines : [{ text: "" }],
                mode: initialMode || "insert"
            });
        }
        
        // Apply initial mode if provided
        if (initialMode && !savedState) {
            this.editorState.next({
                ...this.editorState.getValue(),
                mode: initialMode
            });
        }
        
        // Set up save on state change
        const saveSubscription = this.editorState.subscribe(state => {
            localStorage.setItem(`editorState_${this.id}`, JSON.stringify(state));
        });
        this.subscriptions.push(saveSubscription);

        // Track cursor position changes for navigation history
        let lastTrackedPosition: Position | null = null;
        const historyTrackingSubscription = this.editorState.subscribe(state => {
            const currentPos = state.selection.focus;
            
            if (lastTrackedPosition) {
                const isSignificantJump = Math.abs(lastTrackedPosition.line - currentPos.line) > 5;
                
                if (isSignificantJump) {
                    console.log('Adding to history:', lastTrackedPosition, '->', currentPos);
                    this.addToHistory(lastTrackedPosition);
                }
            }
            
            lastTrackedPosition = {...currentPos};
        });
        this.subscriptions.push(historyTrackingSubscription);

        const keybindingSubscription = keybindingsService.signal.subscribe(s => {

            if (!this.focused.getValue()) {
                return; // Ignore commands if editor is not focused
            }

            clearTimeout(this.cursorBlinkRestoreTimeout);

            const currentState = this.editorState.getValue();

            // Check for Konami code arrow keys in macro editor
            if (this.id === 'macro' && currentState.mode === "insert") {
                let konamiKey: string | null = null;
                switch (s) {
                    case "editor.moveup":
                        konamiKey = 'ArrowUp';
                        break;
                    case "editor.movedown":
                        konamiKey = 'ArrowDown';
                        break;
                    case "editor.moveleft":
                        konamiKey = 'ArrowLeft';
                        break;
                    case "editor.moveright":
                        konamiKey = 'ArrowRight';
                        break;
                }
                
                if (konamiKey && this.checkKonamiCode(konamiKey)) {
                    // Konami code completed!
                    this.triggerEasterEgg(currentState);
                    return;
                }
            }

            switch (s) {
                case "editor.undo":
                    this.editorState.next(this.undoRedo.undo(currentState));
                    break;
                case "editor.redo":
                    this.editorState.next(this.undoRedo.redo(currentState));
                    break;
                case "editor.clearHistory":
                    this.undoRedo.clear();
                    break;
                case "editor.moveright":
                    this.moveRight();
                    break;
                case "editor.moveleft":
                    this.moveLeft();
                    break;
                case "editor.moveup":
                    this.moveUp();
                    break;
                case "editor.movedown":
                    this.moveDown();
                    break;

                case "editor.copy":
                    this.copy();
                    break;
                case "editor.cut":
                    this.cut();
                    break;
                case "editor.paste":
                    this.paste();
                    break;

                case "editor.selectall":
                    this.selectAll();
                    break;

                case "editor.search":
                    this.searchStore.show();
                    break;

                case "editor.quicknav":
                    this.quickNavStore.show();
                    break;

                case "editor.selectright":
                    this.updateSelection({
                        line: currentState.selection.focus.line,
                        column: currentState.selection.focus.column + 1
                    });
                    break;
                case "editor.selectleft":
                    this.updateSelection({
                        line: currentState.selection.focus.line,
                        column: currentState.selection.focus.column - 1
                    });
                    break;
                case "editor.selectup":
                    this.updateSelection({
                        line: currentState.selection.focus.line - 1,
                        column: currentState.selection.focus.column
                    });
                    break;
                case "editor.selectdown":
                    this.updateSelection({
                        line: currentState.selection.focus.line + 1,
                        column: currentState.selection.focus.column
                    });
                    break;

                case "editor.selectwordleft":
                    this.selectToTheBeginningOfWord();
                    break;
                case "editor.selectwordright":
                    this.selectToTheEndOfWord();
                    break;
                    
                case "editor.movelinestart":
                    this.moveToLineStart();
                    break;
                    
                case "editor.movelineend":
                    this.moveToLineEnd();
                    break;
                    
                case "editor.selectlinestart":
                    this.selectToLineStart();
                    break;
                    
                case "editor.selectlineend":
                    this.selectToLineEnd();
                    break;

                case "editor.togglecomment":
                    this.toggleComment();
                    break;

                case "editor.navigateback":
                    console.log('Navigate back command triggered');
                    this.navigateToPreviousCursor();
                    break;

                case "editor.navigateforward":
                    console.log('Navigate forward command triggered');
                    this.navigateToNextCursor();
                    break;

                default:
                    console.warn(`Unknown command: ${s}`);
            }

            this.cursorBlinkState.next(false);

            this.cursorBlinkRestoreTimeout = window.setTimeout(() => {
                this.cursorBlinkState.next(true);
            }, 500);
        });
        this.subscriptions.push(keybindingSubscription);

        const keypressSubscription = keybindingsService.keypressSignal.subscribe(event => {
            const currentState = this.editorState.getValue();
            const selection = currentState.selection;

            if (!this.focused.getValue()) {
                return; // Ignore commands if editor is not focused
            }

            // Check for Konami code letter keys in macro editor
            if (this.id === 'macro' && currentState.mode === "insert") {
                // Only check for 'b' and 'a' keys here, arrow keys are handled in keybinding subscription
                if ((event.key === 'b' || event.key === 'a') && this.checkKonamiCode(event.key)) {
                    // Konami code completed!
                    this.triggerEasterEgg(currentState);
                    return;
                }
            }

            if (currentState.mode === "insert") {
                const char = event.key;

                // Handle selection deletion first
                if (!isSelectionCollapsed(selection) && (char.length === 1 || char === "Enter")) {
                    // Delete the selection first
                    const range = selectionToRange(selection);
                    const deletedText = CommandExecutor.extractText(range, currentState);

                    const deleteCommand: CommandData = {
                        type: "delete",
                        range,
                        deletedText
                    };

                    const stateAfterDelete = this.undoRedo.execute(deleteCommand, currentState);

                    // Then insert the new character
                    if (char.length === 1 || char === "Enter") {
                        const insertCommand: CommandData = {
                            type: "insert",
                            position: range.start,
                            text: char === "Enter" ? "\n" : char
                        };

                        this.editorState.next(this.undoRedo.execute(insertCommand, stateAfterDelete));
                    } else {
                        this.editorState.next(stateAfterDelete);
                    }
                    return;
                }

                if (char.length === 1) {
                    const command: CommandData = {
                        type: "insert",
                        position: selection.focus,
                        text: char
                    };
                    this.editorState.next(this.undoRedo.execute(command, currentState));
                } else if (char === "Backspace") {
                    let range: Range;

                    if (!isSelectionCollapsed(selection)) {
                        // Delete selection
                        range = selectionToRange(selection);
                    } else if (selection.focus.column > 0) {
                        // Delete character before cursor
                        range = {
                            start: {
                                line: selection.focus.line,
                                column: selection.focus.column - 1
                            },
                            end: selection.focus
                        };
                    } else if (selection.focus.line > 0) {
                        // At start of line, delete newline
                        const prevLine = currentState.lines[selection.focus.line - 1];
                        range = {
                            start: {line: selection.focus.line - 1, column: prevLine.text.length},
                            end: selection.focus
                        };
                    } else {
                        // Nothing to delete
                        return;
                    }

                    const deletedText = CommandExecutor.extractText(range, currentState);

                    const command: CommandData = {
                        type: "delete",
                        range,
                        deletedText
                    };

                    this.editorState.next(this.undoRedo.execute(command, currentState));
                } else if (char === "Enter") {
                    const command: CommandData = {
                        type: "insert",
                        position: selection.focus,
                        text: "\n"
                    };

                    this.editorState.next(this.undoRedo.execute(command, currentState));
                } else if (char === "Tab") {
                    // Check if shift is held for shift+tab
                    this.handleTab(event.shiftKey);
                }
            } else if (currentState.mode === "command") {
                console.log(`Command mode input: ${event.key}`);
            }
        });
        this.subscriptions.push(keypressSubscription);
    }

    public setMode(mode: "normal" | "insert" | "command") {
        this.editorState.next({
            ...this.editorState.getValue(),
            mode
        });
    }

    public moveRight() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        const oldFocus = {...selection.focus};

        // Move the focus to the right
        const newFocus = {
            line: selection.focus.line,
            column: selection.focus.column + 1
        };

        // If the focus goes beyond the line length, move to next line
        if (newFocus.column > currentState.lines[newFocus.line].text.length) {
            if (newFocus.line < currentState.lines.length - 1) {
                newFocus.line++;
                newFocus.column = 0;
            } else {
                return; // Can't move further
            }
        }

        const command: CommandData = {
            type: "move",
            from: oldFocus,
            to: newFocus
        }

        this.editorState.next(this.undoRedo.execute(command, currentState));
    }

    public moveLeft() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        const oldFocus = {...selection.focus};

        // Move the focus to the left
        const newFocus = {
            line: selection.focus.line,
            column: selection.focus.column - 1
        };

        // If the focus goes before the start of the line, move to previous line
        if (newFocus.column < 0) {
            if (newFocus.line > 0) {
                newFocus.line--;
                newFocus.column = currentState.lines[newFocus.line].text.length;
            } else {
                return; // Can't move further
            }
        }

        const command: CommandData = {
            type: "move",
            from: oldFocus,
            to: newFocus
        }

        this.editorState.next(this.undoRedo.execute(command, currentState));
    }

    public moveUp() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        const oldFocus = {...selection.focus};

        // Move the focus up
        const newFocus = {
            line: selection.focus.line - 1,
            column: selection.focus.column
        };

        if (newFocus.line < 0) {
            return;
        } else if (newFocus.column > currentState.lines[newFocus.line].text.length) {
            newFocus.column = currentState.lines[newFocus.line].text.length;
        }

        const command: CommandData = {
            type: "move",
            from: oldFocus,
            to: newFocus
        }

        this.editorState.next(this.undoRedo.execute(command, currentState));
    }

    public moveDown() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        const oldFocus = {...selection.focus};

        // Move the focus down
        const newFocus = {
            line: selection.focus.line + 1,
            column: selection.focus.column
        };

        // If the focus goes beyond the last line, keep it at the last line
        if (newFocus.line >= currentState.lines.length) {
            return;
        } else if (newFocus.column > currentState.lines[newFocus.line].text.length) {
            newFocus.column = currentState.lines[newFocus.line].text.length;
        }

        const command: CommandData = {
            type: "move",
            from: oldFocus,
            to: newFocus
        }

        this.editorState.next(this.undoRedo.execute(command, currentState));
    }

    public setCursorPosition(position: Position) {
        const currentState = this.editorState.getValue();

        // Validate position
        if (position.line < 0 || position.line >= currentState.lines.length) {
            return;
        }

        const line = currentState.lines[position.line];
        if (position.column < 0 || position.column > line.text.length) {
            position.column = Math.max(0, Math.min(position.column, line.text.length));
        }

        // Create a move command
        const command: CommandData = {
            type: "move",
            from: currentState.selection.focus,
            to: position
        };

        // Execute the command (this updates selection to have anchor = focus = position)
        this.editorState.next(this.undoRedo.execute(command, currentState));
    }

    public startSelection(position: Position) {
        const currentState = this.editorState.getValue();

        // Validate position
        if (position.line < 0 || position.line >= currentState.lines.length) {
            return;
        }

        const line = currentState.lines[position.line];
        if (position.column < 0 || position.column > line.text.length) {
            position.column = Math.max(0, Math.min(position.column, line.text.length));
        }

        // Set both anchor and focus to the same position (collapsed selection)
        this.editorState.next({
            ...currentState,
            selection: {
                anchor: position,
                focus: position
            }
        });
    }

    public updateSelection(position: Position) {
        const currentState = this.editorState.getValue();

        // Validate position
        if (position.line < 0 || position.line >= currentState.lines.length) {
            return;
        }

        const line = currentState.lines[position.line];
        if (position.column < 0 || position.column > line.text.length) {
            position.column = Math.max(0, Math.min(position.column, line.text.length));
        }

        // Keep anchor, update focus
        this.editorState.next({
            ...currentState,
            selection: {
                ...currentState.selection,
                focus: position
            }
        });
    }

    public selectWord(position: Position) {
        const currentState = this.editorState.getValue();

        if (position.line < 0 || position.line >= currentState.lines.length) {
            return;
        }

        const line = currentState.lines[position.line].text;
        const col = Math.min(position.column, line.length - 1);

        // Find word boundaries
        const wordRegex = /\w+/g;
        let match;
        let wordStart = col;
        let wordEnd = col;

        while ((match = wordRegex.exec(line)) !== null) {
            if (match.index <= col && match.index + match[0].length > col) {
                wordStart = match.index;
                wordEnd = match.index + match[0].length;
                break;
            }
        }

        // If not on a word, select the character
        if (wordStart === wordEnd) {
            wordEnd = Math.min(col + 1, line.length);
        }

        this.editorState.next({
            ...currentState,
            selection: {
                anchor: { line: position.line, column: wordStart },
                focus: { line: position.line, column: wordEnd }
            }
        });
    }

    public moveToLineStart() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;
        
        const newPosition = {
            line: selection.focus.line,
            column: 0
        };
        
        const command: CommandData = {
            type: "move",
            from: selection.focus,
            to: newPosition
        };
        
        this.editorState.next(this.undoRedo.execute(command, currentState));
    }
    
    public moveToLineEnd() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;
        const lineLength = currentState.lines[selection.focus.line].text.length;
        
        const newPosition = {
            line: selection.focus.line,
            column: lineLength
        };
        
        const command: CommandData = {
            type: "move",
            from: selection.focus,
            to: newPosition
        };
        
        this.editorState.next(this.undoRedo.execute(command, currentState));
    }
    
    public selectToLineStart() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;
        
        const newFocus = {
            line: selection.focus.line,
            column: 0
        };
        
        this.editorState.next({
            ...currentState,
            selection: {
                anchor: selection.anchor,
                focus: newFocus
            }
        });
    }
    
    public selectToLineEnd() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;
        const lineLength = currentState.lines[selection.focus.line].text.length;
        
        const newFocus = {
            line: selection.focus.line,
            column: lineLength
        };
        
        this.editorState.next({
            ...currentState,
            selection: {
                anchor: selection.anchor,
                focus: newFocus
            }
        });
    }

    private selectToTheBeginningOfWord() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        const range = selectionToRange(selection);
        const line = currentState.lines[range.start.line];
        if (!line) return;

        const lineText = line.text;
        const currentStart = range.start.column;

        // If at beginning of line, move to previous line
        if (currentStart === 0) {
            if (range.start.line > 0) {
                // Move to end of previous line
                const prevLineLength = currentState.lines[range.start.line - 1].text.length;

                this.editorState.next({
                    ...currentState,
                    selection: {
                        anchor: selection.anchor,
                        focus: {
                            line: range.start.line - 1,
                            column: prevLineLength
                        }
                    }
                });
            }
            return;
        }

        // Get the character BEFORE current start position
        const charBefore = lineText[currentStart - 1];
        let newStart = currentStart;

        // Define expansion rules (working backwards)
        if (/\s/.test(charBefore)) {
            // Before whitespace: skip all whitespace backwards
            newStart--;
            while (newStart > 0 && /\s/.test(lineText[newStart - 1])) {
                newStart--;
            }
        } else if ('[]'.includes(charBefore)) {
            // Brackets: expand by exactly one
            newStart--;
        } else if ('+-'.includes(charBefore)) {
            // Arithmetic: expand to include all consecutive arithmetic ops
            newStart--;
            while (newStart > 0 && '+-'.includes(lineText[newStart - 1])) {
                newStart--;
            }
        } else if ('><'.includes(charBefore)) {
            // Pointer ops: expand to include all consecutive pointer ops
            newStart--;
            while (newStart > 0 && '><'.includes(lineText[newStart - 1])) {
                newStart--;
            }
        } else if ('.,'.includes(charBefore)) {
            // I/O: expand backwards through consecutive I/O
            newStart--;
            while (newStart > 0 && '.,'.includes(lineText[newStart - 1])) {
                newStart--;
            }
        } else {
            // Comments/other: expand backwards to next brainfuck token or whitespace
            newStart--;
            while (newStart > 0 &&
            !'+-><[].,$'.includes(lineText[newStart - 1]) &&
            !/\s/.test(lineText[newStart - 1])) {
                newStart--;
            }
        }

        // Update selection - contract from current start
        this.editorState.next({
            ...currentState,
            selection: {
                anchor: selection.anchor,
                focus: { line: range.start.line, column: newStart }
            }
        });
    }

    private selectToTheEndOfWord() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        const range = selectionToRange(selection);
        const line = currentState.lines[range.end.line];
        if (!line) return;

        const lineText = line.text;
        const currentEnd = range.end.column;

        // If at end of line, move to next line
        if (currentEnd >= lineText.length) {
            if (range.end.line < currentState.lines.length - 1) {
                // Find first non-whitespace on next line
                const nextLine = currentState.lines[range.end.line + 1].text;
                let col = 0;
                while (col < nextLine.length && /\s/.test(nextLine[col])) {
                    col++;
                }

                this.editorState.next({
                    ...currentState,
                    selection: {
                        anchor: selection.anchor,
                        focus: { line: range.end.line + 1, column: col }
                    }
                });
            }
            return;
        }

        // Get the character at current end position
        const char = lineText[currentEnd];
        let newEnd = currentEnd;

        // Define expansion rules
        if (/\s/.test(char)) {
            // On whitespace: skip all whitespace, then stop at next token
            while (newEnd < lineText.length && /\s/.test(lineText[newEnd])) {
                newEnd++;
            }
            // Include one more character if not at end
            if (newEnd < lineText.length) {
                newEnd++;
            }
        } else if ('[]'.includes(char)) {
            // Brackets: expand by exactly one
            newEnd++;
        } else if ('+-'.includes(char)) {
            // Arithmetic: expand to include all consecutive arithmetic ops
            while (newEnd < lineText.length && '+-'.includes(lineText[newEnd])) {
                newEnd++;
            }
        } else if ('><'.includes(char)) {
            // Pointer ops: expand to include all consecutive pointer ops
            while (newEnd < lineText.length && '><'.includes(lineText[newEnd])) {
                newEnd++;
            }
        } else if ('.,'.includes(char)) {
            // I/O: usually single, but expand if multiple
            while (newEnd < lineText.length && '.,'.includes(lineText[newEnd])) {
                newEnd++;
            }
        } else {
            // Comments/other: expand to next brainfuck token or whitespace
            while (newEnd < lineText.length &&
            !'+-><[].,$'.includes(lineText[newEnd]) &&
            !/\s/.test(lineText[newEnd])) {
                newEnd++;
            }
        }

        // Update selection - expand from current end
        this.editorState.next({
            ...currentState,
            selection: {
                anchor: selection.anchor,
                focus: { line: range.end.line, column: newEnd }
            }
        });
    }
    public selectLine(lineNumber: number) {
        const currentState = this.editorState.getValue();

        if (lineNumber < 0 || lineNumber >= currentState.lines.length) {
            return;
        }

        const lineLength = currentState.lines[lineNumber].text.length;

        this.editorState.next({
            ...currentState,
            selection: {
                anchor: { line: lineNumber, column: 0 },
                focus: { line: lineNumber, column: lineLength }
            }
        });
    }

    public selectAll() {
        const currentState = this.editorState.getValue();
        const lastLineIndex = currentState.lines.length - 1;
        const lastLineLength = currentState.lines[lastLineIndex].text.length;

        this.editorState.next({
            ...currentState,
            selection: {
                anchor: { line: 0, column: 0 },
                focus: { line: lastLineIndex, column: lastLineLength }
            }
        });
    }

    public copy() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        if (isSelectionCollapsed(selection)) {
            return; // Nothing to copy
        }

        const range = selectionToRange(selection);
        const textToCopy = CommandExecutor.extractText(range, currentState);

        navigator.clipboard.writeText(textToCopy).catch(err => {
            console.error("Failed to copy text:", err);
        });
    }

    public cut() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        if (isSelectionCollapsed(selection)) {
            // Cut the entire line when selection is collapsed
            const lineIndex = selection.focus.line;
            const line = currentState.lines[lineIndex];
            if (!line) return;

            // Include the newline character if not the last line
            const isLastLine = lineIndex === currentState.lines.length - 1;
            const textToCut = line.text + (isLastLine ? '' : '\n');

            navigator.clipboard.writeText(textToCut).then(() => {
                // Delete the entire line
                const range: Range = {
                    start: { line: lineIndex, column: 0 },
                    end: isLastLine 
                        ? { line: lineIndex, column: line.text.length }
                        : { line: lineIndex + 1, column: 0 }
                };

                const deleteCommand: CommandData = {
                    type: "delete",
                    range,
                    deletedText: textToCut
                };
                this.editorState.next(this.undoRedo.execute(deleteCommand, currentState));
            }).catch(err => {
                console.error("Failed to cut line:", err);
            });
            return;
        }

        const range = selectionToRange(selection);
        const textToCut = CommandExecutor.extractText(range, currentState);

        navigator.clipboard.writeText(textToCut).then(() => {
            // Now delete the selected text
            const deleteCommand: CommandData = {
                type: "delete",
                range,
                deletedText: textToCut
            };
            this.editorState.next(this.undoRedo.execute(deleteCommand, currentState));
        }).catch(err => {
            console.error("Failed to cut text:", err);
        });
    }

    public paste() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;

        navigator.clipboard.readText().then(text => {
            if (text.length === 0) {
                return; // Nothing to paste
            }

            let stateForInsert = currentState;
            
            // If there's a selection, delete it first
            if (!isSelectionCollapsed(selection)) {
                const range = selectionToRange(selection);
                const deleteCommand: CommandData = {
                    type: "delete",
                    range,
                    deletedText: CommandExecutor.extractText(range, currentState)
                };
                stateForInsert = this.undoRedo.execute(deleteCommand, currentState);
                this.editorState.next(stateForInsert);
            }

            // Now insert the text at the cursor position from the updated state
            const insertCommand: CommandData = {
                type: "insert",
                position: stateForInsert.selection.focus,
                text
            };
            this.editorState.next(this.undoRedo.execute(insertCommand, stateForInsert));
        }).catch(err => {
            console.error("Failed to read clipboard:", err);
        });
    }

    public toggleComment() {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;
        const range = selectionToRange(selection);
        
        // Get all lines that are part of the selection
        const startLine = range.start.line;
        const endLine = range.end.line;
        
        // Check if all selected lines are already commented
        let allLinesCommented = true;
        for (let i = startLine; i <= endLine; i++) {
            const line = currentState.lines[i];
            if (!line) continue;
            
            const trimmedLine = line.text.trim();
            if (trimmedLine.length > 0 && !trimmedLine.startsWith('//')) {
                allLinesCommented = false;
                break;
            }
        }
        
        const commands: CommandData[] = [];
        
        for (let lineIdx = startLine; lineIdx <= endLine; lineIdx++) {
            const line = currentState.lines[lineIdx];
            if (!line) continue;
            
            if (allLinesCommented) {
                // Remove comment from line
                const commentIndex = line.text.indexOf('//');
                if (commentIndex !== -1) {
                    // Remove '//' and one space after it if present
                    const deleteEnd = line.text[commentIndex + 2] === ' ' ? commentIndex + 3 : commentIndex + 2;
                    commands.push({
                        type: "delete",
                        range: {
                            start: { line: lineIdx, column: commentIndex },
                            end: { line: lineIdx, column: deleteEnd }
                        },
                        deletedText: line.text.slice(commentIndex, deleteEnd)
                    });
                }
            } else {
                // Add comment to line (skip empty lines)
                if (line.text.trim().length > 0) {
                    // Find the first non-whitespace character
                    let insertColumn = 0;
                    while (insertColumn < line.text.length && /\s/.test(line.text[insertColumn])) {
                        insertColumn++;
                    }
                    
                    commands.push({
                        type: "insert",
                        position: { line: lineIdx, column: insertColumn },
                        text: "// "
                    });
                }
            }
        }
        
        if (commands.length > 0) {
            const compositeCommand: CommandData = {
                type: "composite",
                commands
            };
            
            // Execute the composite command
            const newState = this.undoRedo.execute(compositeCommand, currentState);
            
            // Check if we're commenting/uncommenting a single line (collapsed selection)
            const isSingleLine = isSelectionCollapsed(selection) || startLine === endLine;
            
            if (isSingleLine && newState.selection.focus.line < newState.lines.length - 1) {
                // Move cursor to the beginning of the next line
                this.editorState.next({
                    ...newState,
                    selection: {
                        anchor: { line: newState.selection.focus.line + 1, column: 0 },
                        focus: { line: newState.selection.focus.line + 1, column: 0 }
                    }
                });
            } else {
                // Maintain selection for multi-line operations
                this.editorState.next({
                    ...newState,
                    selection: currentState.selection
                });
            }
        }
    }

    public getText(): string {
        const state = this.editorState.getValue();
        return state.lines.map(line => line.text).join('\n');
    }

    public getState(): EditorState {
        return this.editorState.getValue();
    }

    public getLines(): Line[] {
        return this.editorState.getValue().lines;
    }

    public focus() {
        this.focused.next(true)
    }

    public blur() {
        this.focused.next(false)
    }

    public clearEditor() {
        this.editorState.next({
            selection: {
                anchor: { line: 0, column: 0 },
                focus: { line: 0, column: 0 }
            },
            lines: [{ text: "" }],
            mode: "insert"
        });
        this.undoRedo.clear();
    }
    
    public setContent(content: string) {
        const lines = content.split('\n').map(text => ({ text }));
        this.editorState.next({
            selection: {
                anchor: { line: 0, column: 0 },
                focus: { line: 0, column: 0 }
            },
            lines: lines.length > 0 ? lines : [{ text: "" }],
            mode: this.editorState.getValue().mode
        });
        this.undoRedo.clear();
    }
    
    public replaceLine(lineIndex: number, newText: string) {
        const state = this.editorState.getValue();
        if (lineIndex < 0 || lineIndex >= state.lines.length) {
            return;
        }
        
        const newLines = [...state.lines];
        newLines[lineIndex] = { text: newText };
        
        this.editorState.next({
            ...state,
            lines: newLines
        });
    }
    
    public replaceRange(start: Position, end: Position, replacement: string) {
        const currentState = this.editorState.getValue();
        const range: Range = { start, end };
        const deletedText = CommandExecutor.extractText(range, currentState);
        
        // Create a composite command for undo/redo
        const commands: CommandData[] = [];
        
        // First delete the range if it's not empty
        if (start.line !== end.line || start.column !== end.column) {
            commands.push({
                type: "delete",
                range,
                deletedText
            });
        }
        
        // Then insert the replacement text
        if (replacement.length > 0) {
            commands.push({
                type: "insert",
                position: start,
                text: replacement
            });
        }
        
        if (commands.length > 0) {
            const command: CommandData = commands.length === 1 
                ? commands[0] 
                : { type: "composite", commands };
                
            this.editorState.next(this.undoRedo.execute(command, currentState));
        }
    }
    
    public getTokenizer(): ITokenizer {
        return this.tokenizer;
    }
    
    public getId(): string {
        return this.id;
    }
    
    public performSearch(query: string) {
        const currentState = this.editorState.getValue();
        const { caseSensitive, wholeWord, useRegex } = this.searchStore.state.value;
        const matches: SearchMatch[] = [];

        if (!query) {
            this.searchStore.setMatches([]);
            return;
        }

        let searchPattern: RegExp;
        try {
            if (useRegex) {
                searchPattern = new RegExp(query, caseSensitive ? 'g' : 'gi');
            } else {
                // Escape special regex characters if not in regex mode
                const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
                const pattern = wholeWord ? `\\b${escapedQuery}\\b` : escapedQuery;
                searchPattern = new RegExp(pattern, caseSensitive ? 'g' : 'gi');
            }
        } catch (e) {
            // Invalid regex, clear matches
            this.searchStore.setMatches([]);
            return;
        }

        // Search through all lines
        currentState.lines.forEach((line, lineIndex) => {
            let match;
            while ((match = searchPattern.exec(line.text)) !== null) {
                matches.push({
                    line: lineIndex,
                    startColumn: match.index,
                    endColumn: match.index + match[0].length
                });
            }
        });

        this.searchStore.setMatches(matches);
    }

    public scrollToCursor() {
        // Force a re-render of the cursor to trigger scrollIntoView
        const currentState = this.editorState.getValue();
        this.editorState.next({
            ...currentState,
            selection: {
                ...currentState.selection,
                // Trigger a change by creating new object references
                focus: { ...currentState.selection.focus },
                anchor: { ...currentState.selection.anchor }
            }
        });
    }

    private checkKonamiCode(key: string): boolean {
        // Clear timer and reset if taking too long between keys
        clearTimeout(this.konamiTimer);
        this.konamiTimer = window.setTimeout(() => {
            this.konamiProgress = 0;
        }, 2000); // 2 second timeout between keys

        // Check if the key matches the expected next key in sequence
        if (key === this.konamiSequence[this.konamiProgress]) {
            this.konamiProgress++;
            
            // Check if complete
            if (this.konamiProgress === this.konamiSequence.length) {
                this.konamiProgress = 0;
                clearTimeout(this.konamiTimer);
                return true;
            }
        } else {
            // Reset if wrong key
            this.konamiProgress = 0;
        }
        
        return false;
    }

    private triggerEasterEgg(currentState: EditorState): void {
        // We need to delete 'b' that was already inserted
        // Since we intercept before 'a' is inserted, we only need to delete the 'b'
        const selection = currentState.selection;
        const focus = selection.focus;
        
        // Check if we have at least 1 character to delete (the 'b')
        if (focus.column >= 1) {
            const range: Range = {
                start: {
                    line: focus.line,
                    column: focus.column - 1
                },
                end: focus
            };
            
            const deletedText = CommandExecutor.extractText(range, currentState);
            
            const deleteCommand: CommandData = {
                type: "delete",
                range,
                deletedText
            };
            
            this.editorState.next(this.undoRedo.execute(deleteCommand, currentState));
        }
        
        // Show the easter egg image
        this.showEasterEggImage();
    }

    private showEasterEggImage(): void {
        // Create overlay
        const overlay = document.createElement('div');
        overlay.style.position = 'fixed';
        overlay.style.top = '0';
        overlay.style.left = '0';
        overlay.style.width = '100vw';
        overlay.style.height = '100vh';
        overlay.style.backgroundColor = 'rgba(0, 0, 0, 0.8)';
        overlay.style.display = 'flex';
        overlay.style.alignItems = 'center';
        overlay.style.justifyContent = 'center';
        overlay.style.zIndex = '9999';
        overlay.style.cursor = 'pointer';
        
        // Create image
        const img = document.createElement('img');
        img.src = '/ee.png';
        img.style.maxWidth = '80%';
        img.style.maxHeight = '80%';
        img.style.borderRadius = '8px';
        img.style.boxShadow = '0 10px 50px rgba(0, 0, 0, 0.5)';
        
        // Add click to close
        overlay.onclick = () => {
            overlay.remove();
        };
        
        // Add escape key to close
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                overlay.remove();
                document.removeEventListener('keydown', handleEscape);
            }
        };
        document.addEventListener('keydown', handleEscape);
        
        overlay.appendChild(img);
        document.body.appendChild(overlay);
        
        // Auto close after 5 seconds
        setTimeout(() => {
            if (overlay.parentNode) {
                overlay.remove();
            }
        }, 5000);
    }

    public destroy(): void {
        // Unsubscribe from all subscriptions
        this.subscriptions.forEach(sub => sub.unsubscribe());
        this.subscriptions = [];
        
        // Complete subjects
        this.editorState.complete();
        this.cursorBlinkState.complete();
        this.focused.complete();
    }
    
    // Batch replace multiple ranges as a single undoable operation
    public batchReplace(replacements: Array<{start: Position, end: Position, text: string}>) {
        const currentState = this.editorState.getValue();
        
        // Sort replacements by position (bottom to top, right to left)
        const sortedReplacements = [...replacements].sort((a, b) => {
            if (a.start.line !== b.start.line) return b.start.line - a.start.line;
            return b.start.column - a.start.column;
        });
        
        // Create composite command
        const commands: CommandData[] = [];
        
        for (const {start, end, text} of sortedReplacements) {
            const range: Range = {start, end};
            const deletedText = CommandExecutor.extractText(range, currentState);
            
            // Delete the old text
            if (start.line !== end.line || start.column !== end.column) {
                commands.push({
                    type: "delete",
                    range,
                    deletedText
                });
            }
            
            // Insert the new text
            if (text.length > 0) {
                commands.push({
                    type: "insert",
                    position: start,
                    text
                });
            }
        }
        
        if (commands.length > 0) {
            const compositeCommand: CommandData = {
                type: "composite",
                commands
            };
            
            // Save current cursor position
            const originalCursor = currentState.selection.focus;
            
            // Execute the composite command
            const newState = this.undoRedo.execute(compositeCommand, currentState);
            
            // Restore cursor position
            const restoredState = {
                ...newState,
                selection: {
                    anchor: originalCursor,
                    focus: originalCursor
                }
            };
            
            this.editorState.next(restoredState);
        }
    }
    
    public handleTab(isShiftTab: boolean) {
        const currentState = this.editorState.getValue();
        const selection = currentState.selection;
        
        if (isSelectionCollapsed(selection)) {
            // No selection - just insert spaces at cursor
            if (!isShiftTab) {
                const command: CommandData = {
                    type: "insert",
                    position: selection.focus,
                    text: "  " // 2 spaces for tab
                };
                this.editorState.next(this.undoRedo.execute(command, currentState));
            } else {
                // For shift+tab with no selection, remove up to 2 spaces before cursor
                const line = currentState.lines[selection.focus.line];
                const textBefore = line.text.slice(0, selection.focus.column);
                
                // Count spaces to remove (up to 2)
                let spacesToRemove = 0;
                for (let i = textBefore.length - 1; i >= Math.max(0, textBefore.length - 2); i--) {
                    if (textBefore[i] === ' ') {
                        spacesToRemove++;
                    } else {
                        break;
                    }
                }
                
                if (spacesToRemove > 0) {
                    const range: Range = {
                        start: {
                            line: selection.focus.line,
                            column: selection.focus.column - spacesToRemove
                        },
                        end: selection.focus
                    };
                    const deletedText = CommandExecutor.extractText(range, currentState);
                    
                    const command: CommandData = {
                        type: "delete",
                        range,
                        deletedText
                    };
                    this.editorState.next(this.undoRedo.execute(command, currentState));
                }
            }
        } else {
            // Selection exists - indent/outdent all lines in selection
            const range = selectionToRange(selection);
            const startLine = range.start.line;
            const endLine = range.end.line;
            
            const commands: CommandData[] = [];
            
            for (let lineIdx = startLine; lineIdx <= endLine; lineIdx++) {
                const line = currentState.lines[lineIdx];
                if (!line) continue;
                
                if (!isShiftTab) {
                    // Add 2 spaces at beginning of line
                    commands.push({
                        type: "insert",
                        position: { line: lineIdx, column: 0 },
                        text: "  "
                    });
                } else {
                    // Remove up to 2 spaces from beginning of line
                    let spacesToRemove = 0;
                    for (let i = 0; i < Math.min(2, line.text.length); i++) {
                        if (line.text[i] === ' ') {
                            spacesToRemove++;
                        } else {
                            break;
                        }
                    }
                    
                    if (spacesToRemove > 0) {
                        const deleteRange: Range = {
                            start: { line: lineIdx, column: 0 },
                            end: { line: lineIdx, column: spacesToRemove }
                        };
                        const deletedText = line.text.slice(0, spacesToRemove);
                        
                        commands.push({
                            type: "delete",
                            range: deleteRange,
                            deletedText
                        });
                    }
                }
            }
            
            if (commands.length > 0) {
                const compositeCommand: CommandData = {
                    type: "composite",
                    commands
                };
                
                // Execute the composite command
                const newState = this.undoRedo.execute(compositeCommand, currentState);
                
                // Adjust selection to maintain the same lines selected
                // Account for the indentation changes
                let newAnchorColumn = selection.anchor.column;
                let newFocusColumn = selection.focus.column;
                
                // Adjust columns based on the operation
                if (selection.anchor.line >= startLine && selection.anchor.line <= endLine) {
                    if (!isShiftTab) {
                        newAnchorColumn += 2;
                    } else {
                        // Calculate how many spaces were actually removed from anchor line
                        const anchorLine = currentState.lines[selection.anchor.line];
                        let spacesRemoved = 0;
                        for (let i = 0; i < Math.min(2, anchorLine.text.length); i++) {
                            if (anchorLine.text[i] === ' ') {
                                spacesRemoved++;
                            } else {
                                break;
                            }
                        }
                        newAnchorColumn = Math.max(0, newAnchorColumn - spacesRemoved);
                    }
                }
                
                if (selection.focus.line >= startLine && selection.focus.line <= endLine) {
                    if (!isShiftTab) {
                        newFocusColumn += 2;
                    } else {
                        // Calculate how many spaces were actually removed from focus line
                        const focusLine = currentState.lines[selection.focus.line];
                        let spacesRemoved = 0;
                        for (let i = 0; i < Math.min(2, focusLine.text.length); i++) {
                            if (focusLine.text[i] === ' ') {
                                spacesRemoved++;
                            } else {
                                break;
                            }
                        }
                        newFocusColumn = Math.max(0, newFocusColumn - spacesRemoved);
                    }
                }
                
                // Update state with adjusted selection
                this.editorState.next({
                    ...newState,
                    selection: {
                        anchor: { line: selection.anchor.line, column: newAnchorColumn },
                        focus: { line: selection.focus.line, column: newFocusColumn }
                    }
                });
            }
        }
    }

    // Cursor position history methods
    private addToHistory(position: Position) {
        // Don't add duplicate positions
        if (this.cursorHistory.length > 0) {
            const lastPos = this.cursorHistory[this.cursorHistory.length - 1];
            if (lastPos.line === position.line && lastPos.column === position.column) {
                return;
            }
        }

        // If we're not at the end of history, truncate future history
        if (this.cursorHistoryIndex < this.cursorHistory.length - 1) {
            this.cursorHistory = this.cursorHistory.slice(0, this.cursorHistoryIndex + 1);
        }

        // Add new position
        this.cursorHistory.push({...position});
        
        // Maintain max size
        if (this.cursorHistory.length > this.MAX_HISTORY_SIZE) {
            this.cursorHistory.shift();
        } else {
            this.cursorHistoryIndex++;
        }
    }

    public navigateToPreviousCursor() {
        console.log(`Navigate back: historyIndex=${this.cursorHistoryIndex}, historyLength=${this.cursorHistory.length}`, this.cursorHistory);
        
        if (this.cursorHistoryIndex <= 0) {
            console.log('No previous position available');
            return; // No previous position
        }

        // Add current position to history if we're at the end
        if (this.cursorHistoryIndex === this.cursorHistory.length - 1) {
            const currentState = this.editorState.getValue();
            console.log('Adding current position to history:', currentState.selection.focus);
            this.addToHistory(currentState.selection.focus);
            this.cursorHistoryIndex--; // Step back one extra since we just added current
        }

        this.cursorHistoryIndex--;
        const targetPosition = this.cursorHistory[this.cursorHistoryIndex];
        console.log('Navigating to position:', targetPosition);
        this.setCursorPosition(targetPosition);
    }

    public navigateToNextCursor() {
        if (this.cursorHistoryIndex >= this.cursorHistory.length - 1) {
            return; // No next position
        }

        this.cursorHistoryIndex++;
        const targetPosition = this.cursorHistory[this.cursorHistoryIndex];
        this.setCursorPosition(targetPosition);
    }
}