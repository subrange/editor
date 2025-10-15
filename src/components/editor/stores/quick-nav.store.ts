import { BehaviorSubject } from 'rxjs';

export interface NavigationItem {
  type: 'macro' | 'mark';
  name: string;
  line: number;
  column: number;
  preview?: string;
}

export interface QuickNavState {
  isVisible: boolean;
  query: string;
  items: NavigationItem[];
  filteredItems: NavigationItem[];
  selectedIndex: number;
}

export class QuickNavStore {
  state: BehaviorSubject<QuickNavState>;

  constructor() {
    this.state = new BehaviorSubject<QuickNavState>({
      isVisible: false,
      query: '',
      items: [],
      filteredItems: [],
      selectedIndex: 0,
    });
  }

  show() {
    this.state.next({
      ...this.state.value,
      isVisible: true,
      query: '',
      selectedIndex: 0,
    });
  }

  hide() {
    this.state.next({
      ...this.state.value,
      isVisible: false,
    });
  }

  setQuery(query: string) {
    const items = this.state.value.items;
    const filteredItems = this.fuzzyFilter(items, query);

    this.state.next({
      ...this.state.value,
      query,
      filteredItems,
      selectedIndex: 0,
    });
  }

  setItems(items: NavigationItem[]) {
    const filteredItems = this.fuzzyFilter(items, this.state.value.query);

    this.state.next({
      ...this.state.value,
      items,
      filteredItems,
      selectedIndex: 0,
    });
  }

  selectNext() {
    const { filteredItems, selectedIndex } = this.state.value;
    if (selectedIndex < filteredItems.length - 1) {
      this.state.next({
        ...this.state.value,
        selectedIndex: selectedIndex + 1,
      });
    }
  }

  selectPrevious() {
    const { selectedIndex } = this.state.value;
    if (selectedIndex > 0) {
      this.state.next({
        ...this.state.value,
        selectedIndex: selectedIndex - 1,
      });
    }
  }

  getSelectedItem(): NavigationItem | null {
    const { filteredItems, selectedIndex } = this.state.value;
    return filteredItems[selectedIndex] || null;
  }

  private fuzzyFilter(
    items: NavigationItem[],
    query: string,
  ): NavigationItem[] {
    if (!query.trim()) {
      return items;
    }

    const lowerQuery = query.toLowerCase();
    const queryChars = lowerQuery.split('');

    return items
      .map((item) => {
        const lowerName = item.name.toLowerCase();
        let score = 0;
        let lastMatchIndex = -1;
        let consecutiveMatches = 0;

        for (let i = 0; i < queryChars.length; i++) {
          const char = queryChars[i];
          const matchIndex = lowerName.indexOf(char, lastMatchIndex + 1);

          if (matchIndex === -1) {
            return null;
          }

          // Bonus for matching at start of word
          if (
            matchIndex === 0 ||
            (matchIndex > 0 && lowerName[matchIndex - 1] === ' ')
          ) {
            score += 10;
          }

          // Bonus for consecutive matches
          if (matchIndex === lastMatchIndex + 1) {
            consecutiveMatches++;
            score += consecutiveMatches * 5;
          } else {
            consecutiveMatches = 0;
          }

          lastMatchIndex = matchIndex;
        }

        // Bonus for exact prefix match
        if (lowerName.startsWith(lowerQuery)) {
          score += 50;
        }

        // Penalty for longer names
        score -= item.name.length * 0.1;

        return { item, score };
      })
      .filter((result) => result !== null)
      .sort((a, b) => b!.score - a!.score)
      .map((result) => result!.item);
  }
}
