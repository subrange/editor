import { BehaviorSubject } from 'rxjs';

export interface EditorDimensions {
    width: number;
    height: number;
}

class DimensionsStore {
    private dimensions$ = new BehaviorSubject<EditorDimensions>({
        width: 1200,
        height: 600
    });

    get state() {
        return this.dimensions$;
    }

    updateDimensions(updates: Partial<EditorDimensions>) {
        this.dimensions$.next({
            ...this.dimensions$.value,
            ...updates
        });
    }

    updateSize(width: number, height: number) {
        this.updateDimensions({ width, height });
    }
}

// Create a singleton instance for each editor
const dimensionsStores = new Map<string, DimensionsStore>();

export function getDimensionsStore(editorId: string): DimensionsStore {
    if (!dimensionsStores.has(editorId)) {
        dimensionsStores.set(editorId, new DimensionsStore());
    }
    return dimensionsStores.get(editorId)!;
}