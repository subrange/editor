import {BehaviorSubject} from "rxjs";
import {keybindingsService} from "../../services/keybindings.service.ts";

export type Position = {
    line: number;
    column: number;
}

export type Selection = {
    anchor: Position;
    focus: Position;
}

type Range = {
    start: Position;
    end: Position;
}

type TextChange = {
    range: Range;
    text: string;
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
                const endPos = this.calculateEndPosition(command.position, command.text);
                return this.executeDelete({
                    type: "delete",
                    range: {
                        start: command.position,
                        end: endPos
                    },
                    deletedText: command.text
                }, state);

            case "delete":
                // Re-insert the deleted text
                if (!command.deletedText) {
                    console.error("Cannot undo delete - no deleted text stored");
                    return state;
                }
                return this.executeInsert({
                    type: "insert",
                    position: command.range.start,
                    text: command.deletedText
                }, state);

            case "move":
                return this.executeMove({
                    type: "move",
                    from: command.to,
                    to: command.from
                }, state);

            case "composite":
                // Undo in reverse order
                return [...command.commands].reverse().reduce((s, cmd) => this.undo(cmd, s), state);
        }
    }

    // Public static method to extract text from a range
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

class EditorStore {
    public editorState = new BehaviorSubject<EditorState>({
        selection: {
            anchor: {line: 0, column: 0},
            focus: {line: 1, column: 3}
        },
        lines: [
            {text: "sldgjk hdsflkjg hsdlkfjgnhsdlkfjghndskfjghnsdklf jnglksdfjnglksdfjngdlkfjng lkjngg lkjgnlksd jnlgsjdn ljdfs ljd ldjs gjlkdfn lgjds nflgjsdnf lgkjsndf lgksdn lkdf"},
            {text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."},
        ],
        mode: "insert"
    });

    public cursorBlinkState = new BehaviorSubject<boolean>(true);
    private cursorBlinkRestoreTimeout: number = 0;

    private undoRedo = new UndoRedo();

    constructor() {
        const savedState = localStorage.getItem("editorState");
        if (savedState) {
            try {
                const parsedState = JSON.parse(savedState);
                this.editorState.next(parsedState);
            } catch (e) {
                console.error("Failed to parse saved editor state:", e);
            }
        }

        // Set up save on state change
        this.editorState.subscribe(state => {
            localStorage.setItem("editorState", JSON.stringify(state));
        });

        keybindingsService.signal.subscribe(s => {
            clearTimeout(this.cursorBlinkRestoreTimeout);

            const currentState = this.editorState.getValue();

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

                default:
                    console.warn(`Unknown command: ${s}`);
            }

            this.cursorBlinkState.next(false);

            this.cursorBlinkRestoreTimeout = window.setTimeout(() => {
                this.cursorBlinkState.next(true);
            }, 500);
        });

        keybindingsService.keypressSignal.subscribe(event => {
            const currentState = this.editorState.getValue();
            const selection = currentState.selection;

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
                }
            } else if (currentState.mode === "command") {
                console.log(`Command mode input: ${event.key}`);
            }
        });
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

    // Add these methods to your EditorStore class:

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
            return; // Nothing to cut
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

            // If there's a selection, delete it first
            if (!isSelectionCollapsed(selection)) {
                const range = selectionToRange(selection);
                const deleteCommand: CommandData = {
                    type: "delete",
                    range,
                    deletedText: CommandExecutor.extractText(range, currentState)
                };
                this.editorState.next(this.undoRedo.execute(deleteCommand, currentState));
            }

            // Now insert the text at the cursor position
            const insertCommand: CommandData = {
                type: "insert",
                position: selection.focus,
                text
            };
            this.editorState.next(this.undoRedo.execute(insertCommand, currentState));
        }).catch(err => {
            console.error("Failed to read clipboard:", err);
        });
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
}

export const editorStore = new EditorStore();