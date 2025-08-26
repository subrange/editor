import { BehaviorSubject } from "rxjs";

export type OutputPosition = 'bottom' | 'right' | 'floating';

interface OutputState {
    collapsed: boolean;
    position: OutputPosition;
    width?: number;  // For right position
    height?: number; // For bottom position
    maxLines?: number; // Optional limit for output lines
}

class OutputStore {
    private loadFromStorage<T>(key: string, defaultValue: T): T {
        try {
            const stored = localStorage.getItem(key);
            return stored !== null ? JSON.parse(stored) : defaultValue;
        } catch {
            return defaultValue;
        }
    }

    private saveToStorage<T>(key: string, value: T) {
        localStorage.setItem(key, JSON.stringify(value));
    }

    state = new BehaviorSubject<OutputState>({
        collapsed: this.loadFromStorage('outputCollapsed', true),
        position: this.loadFromStorage('outputPosition', 'right'),
        width: this.loadFromStorage('outputWidth', 384), // w-96 equivalent
        height: this.loadFromStorage('outputHeight', 384), // h-96 equivalent
        maxLines: this.loadFromStorage('outputMaxLines', 10000)
    });

    setCollapsed(collapsed: boolean) {
        this.state.next({
            ...this.state.value,
            collapsed
        });
        this.saveToStorage('outputCollapsed', collapsed);
    }

    setPosition(position: OutputPosition) {
        this.state.next({
            ...this.state.value,
            position
        });
        this.saveToStorage('outputPosition', position);
    }

    setSize(dimension: 'width' | 'height', value: number) {
        this.state.next({
            ...this.state.value,
            [dimension]: value
        });
        this.saveToStorage(`output${dimension.charAt(0).toUpperCase() + dimension.slice(1)}`, value);
    }

    setMaxLines(maxLines: number | undefined) {
        this.state.next({
            ...this.state.value,
            maxLines
        });
        this.saveToStorage('outputMaxLines', maxLines);
    }
}

export const outputStore = new OutputStore();