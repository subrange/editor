import {BehaviorSubject} from "rxjs";
import {keybindingsService} from "../../services/keybindings.service.ts";

type Position = {
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

type Line = {
    text: string;
}

type EditorState = {
    selection: Selection;
    lines: Line[];
    mode: "normal" | "insert" | "command";
}

type CommandData =
    | { type: "insert"; position: Position; text: string }
    | { type: "delete"; range: Range }
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
                // Delete the inserted text
                // return this.executeDelete({
                //     range: {
                //         start: command.position,
                //         end: this.offsetPosition(command.position, command.text.length)
                //     }
                // }, state);
                return state;
            case "delete":
                // Re-insert the deleted text
                // return this.executeInsert({
                //     position: command.range.start,
                //     text: "fuckoff" // You would need to store the deleted text in the command
                // }, state);
                return state;
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

    private executeInsert(command: { position: Position; text: string }, state: EditorState): EditorState {
        return state;
    }
    //
    private executeDelete(command: { range: Range }, state: EditorState): EditorState {
        return state;
    }
    //
    private executeMove(command: { from: Position; to: Position }, state: EditorState): EditorState {
        const newSelection: Selection = {
            anchor: command.to,
            focus: command.to
        };

        // Update the selection in the state
        return {
            ...state,
            selection: newSelection
        };
    }
}

function positionsEqual(pos1: Position, pos2: Position): boolean {
    return pos1.line === pos2.line && pos1.column === pos2.column;
}

class UndoRedo {
    private history: CommandData[] = [];
    private index: number = -1;
    private executor = new CommandExecutor();

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

        // Merge consecutive inserts at the same position
        // if (last.type === "insert" && current.type === "insert") {
        //     const lastEnd = this.offsetPosition(last.position, last.text.length);
        //     if (this.positionsEqual(lastEnd, current.position)) {
        //         return {
        //             type: "insert",
        //             position: last.position,
        //             text: last.text + current.text
        //         };
        //     }
        // }
        //
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
                default:
                    console.warn(`Unknown command: ${s}`);
            }

            this.cursorBlinkState.next(false);

            this.cursorBlinkRestoreTimeout = window.setTimeout(() => {
                this.cursorBlinkState.next(true);
            }, 500);
        })
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

        // If the focus goes beyond the line length, exit
        if (newFocus.column > currentState.lines[newFocus.line].text.length) {
            return;
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

        // If the focus goes before the start of the line, move to the previous line
        if (newFocus.column < 0) {
            return;
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

    private modState = (state: EditorState, newState: Partial<EditorState>): EditorState => {
        return {
            ...state,
            ...newState,
            selection: {
                ...state.selection,
                ...newState.selection
            },
            lines: newState.lines ? newState.lines : state.lines
        };
    }
}

export const editorStore = new EditorStore();