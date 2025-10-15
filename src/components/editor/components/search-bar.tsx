import { useEffect, useRef } from 'react';
import { useStoreSubscribe } from '../../../hooks/use-store-subscribe.tsx';
import { SearchStore } from '../stores/search.store.ts';
import { EditorStore } from '../stores/editor.store.ts';
import clsx from 'clsx';

interface SearchBarProps {
  searchStore: SearchStore;
  editorStore: EditorStore;
  onSearch: (query: string, jumpToFirst?: boolean) => void;
  onHide?: () => void;
}

export function SearchBar({
  searchStore,
  editorStore,
  onSearch,
  onHide,
}: SearchBarProps) {
  const searchState = useStoreSubscribe(searchStore.state);
  const inputRef = useRef<HTMLInputElement>(null);
  const isNavigatingRef = useRef<boolean>(false);

  // Focus input when search becomes visible
  useEffect(() => {
    if (searchState.isVisible && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [searchState.isVisible]);

  // Handle keyboard events
  useEffect(() => {
    if (!searchState.isVisible) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'Escape':
          e.preventDefault();
          searchStore.hide();
          onHide?.();
          break;
        case 'Enter':
          e.preventDefault();
          isNavigatingRef.current = true;
          if (e.shiftKey) {
            handlePrevious();
          } else {
            handleNext();
          }
          // Reset navigation flag after a short delay
          setTimeout(() => {
            isNavigatingRef.current = false;
          }, 100);
          break;
      }
    };

    // Use capture phase to intercept before editor
    document.addEventListener('keydown', handleKeyDown, true);
    return () => document.removeEventListener('keydown', handleKeyDown, true);
  }, [searchState.isVisible, searchStore, onHide]);

  if (!searchState.isVisible) {
    return null;
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const query = e.target.value;

    // Don't jump to first match if we're just navigating
    if (isNavigatingRef.current) {
      return;
    }

    searchStore.setQuery(query);
    onSearch(query, true); // Jump to first match when typing
  };

  const handleNext = () => {
    isNavigatingRef.current = true;
    searchStore.nextMatch();
    const match = searchStore.getCurrentMatch();
    if (match) {
      editorStore.setCursorPosition({
        line: match.line,
        column: match.startColumn,
      });
    }
    setTimeout(() => {
      isNavigatingRef.current = false;
    }, 100);
  };

  const handlePrevious = () => {
    isNavigatingRef.current = true;
    searchStore.previousMatch();
    const match = searchStore.getCurrentMatch();
    if (match) {
      editorStore.setCursorPosition({
        line: match.line,
        column: match.startColumn,
      });
    }
    setTimeout(() => {
      isNavigatingRef.current = false;
    }, 100);
  };

  const matchInfo =
    searchState.matches.length > 0
      ? `${searchState.currentMatchIndex + 1} of ${searchState.matches.length}`
      : searchState.query
        ? 'No results'
        : '';

  return (
    <div className="absolute top-2 left-2 h-12 z-50 bg-zinc-900 border border-zinc-700 rounded-md shadow-lg p-2 flex items-center gap-2">
      <input
        ref={inputRef}
        type="text"
        value={searchState.query}
        onChange={handleInputChange}
        placeholder="Search..."
        className="bg-zinc-800 text-zinc-100 px-2 py-1 rounded text-sm outline-none focus:ring-1 focus:ring-blue-500 w-48"
        onMouseDown={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          // Prevent default Enter behavior to avoid form submission or other side effects
          if (e.key === 'Enter' || e.key === 'Escape') {
            e.preventDefault();
          }
        }}
      />

      <span className="text-xs text-zinc-500 min-w-[80px] text-center">
        {matchInfo}
      </span>

      <div className="flex items-center gap-1">
        <button
          onClick={handlePrevious}
          disabled={searchState.matches.length === 0}
          className={clsx(
            'px-2 py-1 text-xs rounded hover:bg-zinc-800',
            searchState.matches.length === 0 && 'opacity-50 cursor-not-allowed',
          )}
          title="Previous match (Shift+Enter)"
        >
          ↑
        </button>
        <button
          onClick={handleNext}
          disabled={searchState.matches.length === 0}
          className={clsx(
            'px-2 py-1 text-xs rounded hover:bg-zinc-800',
            searchState.matches.length === 0 && 'opacity-50 cursor-not-allowed',
          )}
          title="Next match (Enter)"
        >
          ↓
        </button>
      </div>

      <div className="flex items-center gap-1 ml-2">
        <button
          onClick={() => {
            searchStore.toggleCaseSensitive();
            onSearch(searchState.query);
          }}
          className={clsx(
            'px-2 py-1 text-xs rounded',
            searchState.caseSensitive
              ? 'bg-blue-600 text-white'
              : 'hover:bg-zinc-800',
          )}
          title="Case sensitive"
        >
          Aa
        </button>
        <button
          onClick={() => {
            searchStore.toggleWholeWord();
            onSearch(searchState.query);
          }}
          className={clsx(
            'px-2 py-1 text-xs rounded',
            searchState.wholeWord
              ? 'bg-blue-600 text-white'
              : 'hover:bg-zinc-800',
          )}
          title="Whole word"
        >
          W
        </button>
        <button
          onClick={() => {
            searchStore.toggleRegex();
            onSearch(searchState.query);
          }}
          className={clsx(
            'px-2 py-1 text-xs rounded',
            searchState.useRegex
              ? 'bg-blue-600 text-white'
              : 'hover:bg-zinc-800',
          )}
          title="Regular expression"
        >
          .*
        </button>
      </div>

      <button
        onClick={() => {
          searchStore.hide();
          onHide?.();
        }}
        className="px-2 py-1 text-xs rounded hover:bg-zinc-800 ml-2"
        title="Close (Escape)"
      >
        ✕
      </button>
    </div>
  );
}
