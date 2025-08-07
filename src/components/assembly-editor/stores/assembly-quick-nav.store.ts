import { BehaviorSubject } from 'rxjs';

export interface AssemblyNavigationItem {
    type: 'label' | 'mark';
    name: string;
    line: number;
    column: number;
}

export interface AssemblyQuickNavState {
    isVisible: boolean;
    query: string;
    items: AssemblyNavigationItem[];
    filteredItems: AssemblyNavigationItem[];
    selectedIndex: number;
}

export class AssemblyQuickNavStore {
    private readonly initialState: AssemblyQuickNavState = {
        isVisible: false,
        query: '',
        items: [],
        filteredItems: [],
        selectedIndex: 0
    };

    public readonly state = new BehaviorSubject<AssemblyQuickNavState>(this.initialState);

    show(items: AssemblyNavigationItem[]) {
        this.state.next({
            ...this.state.value,
            isVisible: true,
            items,
            filteredItems: items,
            query: '',
            selectedIndex: 0
        });
    }

    hide() {
        this.state.next({
            ...this.initialState,
            isVisible: false
        });
    }

    setQuery(query: string) {
        const filteredItems = this.state.value.items.filter(item =>
            item.name.toLowerCase().includes(query.toLowerCase())
        );

        this.state.next({
            ...this.state.value,
            query,
            filteredItems,
            selectedIndex: 0
        });
    }

    selectNext() {
        const { selectedIndex, filteredItems } = this.state.value;
        if (selectedIndex < filteredItems.length - 1) {
            this.state.next({
                ...this.state.value,
                selectedIndex: selectedIndex + 1
            });
        }
    }

    selectPrevious() {
        const { selectedIndex } = this.state.value;
        if (selectedIndex > 0) {
            this.state.next({
                ...this.state.value,
                selectedIndex: selectedIndex - 1
            });
        }
    }

    getSelectedItem(): AssemblyNavigationItem | null {
        const { selectedIndex, filteredItems } = this.state.value;
        return filteredItems[selectedIndex] || null;
    }
}