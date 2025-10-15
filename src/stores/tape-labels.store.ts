import { BehaviorSubject } from 'rxjs';

export interface TapeLabels {
  lanes: Record<number, string>; // lane index -> label
  columns: Record<number, string>; // column index -> label
  cells: Record<number, string>; // cell index -> label
}

class TapeLabelsStore {
  private readonly STORAGE_KEY = 'tapeLabels';

  public labels = new BehaviorSubject<TapeLabels>({
    lanes: this.loadFromStorage().lanes || {},
    columns: this.loadFromStorage().columns || {},
    cells: this.loadFromStorage().cells || {},
  });

  setLaneLabel(index: number, label: string) {
    const current = this.labels.value;
    const newLabels = {
      ...current,
      lanes: {
        ...current.lanes,
        [index]: label,
      },
    };
    this.labels.next(newLabels);
    this.saveToStorage(newLabels);
  }

  setColumnLabel(index: number, label: string) {
    const current = this.labels.value;
    const newLabels = {
      ...current,
      columns: {
        ...current.columns,
        [index]: label,
      },
    };
    this.labels.next(newLabels);
    this.saveToStorage(newLabels);
  }

  removeLaneLabel(index: number) {
    const current = this.labels.value;
    const newLanes = { ...current.lanes };
    delete newLanes[index];
    const newLabels = {
      ...current,
      lanes: newLanes,
    };
    this.labels.next(newLabels);
    this.saveToStorage(newLabels);
  }

  removeColumnLabel(index: number) {
    const current = this.labels.value;
    const newColumns = { ...current.columns };
    delete newColumns[index];
    const newLabels = {
      ...current,
      columns: newColumns,
    };
    this.labels.next(newLabels);
    this.saveToStorage(newLabels);
  }

  setCellLabel(index: number, label: string) {
    const current = this.labels.value;
    const newLabels = {
      ...current,
      cells: {
        ...current.cells,
        [index]: label,
      },
    };
    this.labels.next(newLabels);
    this.saveToStorage(newLabels);
  }

  removeCellLabel(index: number) {
    const current = this.labels.value;
    const newCells = { ...current.cells };
    delete newCells[index];
    const newLabels = {
      ...current,
      cells: newCells,
    };
    this.labels.next(newLabels);
    this.saveToStorage(newLabels);
  }

  clearAllLabels() {
    const newLabels = {
      lanes: {},
      columns: {},
      cells: {},
    };
    this.labels.next(newLabels);
    this.saveToStorage(newLabels);
  }

  private loadFromStorage(): TapeLabels {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      return stored
        ? JSON.parse(stored)
        : { lanes: {}, columns: {}, cells: {} };
    } catch {
      return { lanes: {}, columns: {}, cells: {} };
    }
  }

  private saveToStorage(labels: TapeLabels) {
    localStorage.setItem(this.STORAGE_KEY, JSON.stringify(labels));
  }
}

export const tapeLabelsStore = new TapeLabelsStore();
