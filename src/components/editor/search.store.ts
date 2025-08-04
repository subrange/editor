import { BehaviorSubject } from "rxjs";
import type { Position } from "./editor.store";

export interface SearchMatch {
    line: number;
    startColumn: number;
    endColumn: number;
}

interface SearchState {
    isVisible: boolean;
    query: string;
    matches: SearchMatch[];
    currentMatchIndex: number;
    caseSensitive: boolean;
    wholeWord: boolean;
    useRegex: boolean;
    scrollTrigger: number;
}

export class SearchStore {
    private readonly initialState: SearchState = {
        isVisible: false,
        query: "",
        matches: [],
        currentMatchIndex: -1,
        caseSensitive: false,
        wholeWord: false,
        useRegex: false,
        scrollTrigger: 0,
    };

    public readonly state = new BehaviorSubject<SearchState>(this.initialState);

    public show() {
        this.state.next({
            ...this.state.value,
            isVisible: true,
        });
    }

    public hide() {
        this.state.next({
            ...this.state.value,
            isVisible: false,
            matches: [],
            currentMatchIndex: -1,
        });
    }

    public setQuery(query: string) {
        this.state.next({
            ...this.state.value,
            query,
        });
    }

    public setMatches(matches: SearchMatch[]) {
        this.state.next({
            ...this.state.value,
            matches,
            currentMatchIndex: matches.length > 0 ? 0 : -1,
            scrollTrigger: this.state.value.scrollTrigger + 1,
        });
    }

    public nextMatch() {
        const { matches, currentMatchIndex } = this.state.value;
        if (matches.length === 0) return;

        const nextIndex = (currentMatchIndex + 1) % matches.length;
        this.state.next({
            ...this.state.value,
            currentMatchIndex: nextIndex,
        });
    }

    public previousMatch() {
        const { matches, currentMatchIndex } = this.state.value;
        if (matches.length === 0) return;

        const prevIndex = currentMatchIndex <= 0 ? matches.length - 1 : currentMatchIndex - 1;
        this.state.next({
            ...this.state.value,
            currentMatchIndex: prevIndex,
        });
    }

    public toggleCaseSensitive() {
        this.state.next({
            ...this.state.value,
            caseSensitive: !this.state.value.caseSensitive,
        });
    }

    public toggleWholeWord() {
        this.state.next({
            ...this.state.value,
            wholeWord: !this.state.value.wholeWord,
        });
    }

    public toggleRegex() {
        this.state.next({
            ...this.state.value,
            useRegex: !this.state.value.useRegex,
        });
    }

    public getCurrentMatch(): SearchMatch | null {
        const { matches, currentMatchIndex } = this.state.value;
        if (currentMatchIndex < 0 || currentMatchIndex >= matches.length) {
            return null;
        }
        return matches[currentMatchIndex];
    }

    public getMatchPosition(index: number): Position | null {
        const { matches } = this.state.value;
        if (index < 0 || index >= matches.length) {
            return null;
        }
        const match = matches[index];
        return { line: match.line, column: match.startColumn };
    }
}