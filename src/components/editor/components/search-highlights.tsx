import { useStoreSubscribeToField } from '../../../hooks/use-store-subscribe.tsx';
import { SearchStore, type SearchMatch } from '../stores/search.store.ts';
import {
  CHAR_HEIGHT,
  LINE_PADDING_LEFT,
  LINE_PADDING_TOP,
} from '../constants.ts';
import clsx from 'clsx';

interface SearchHighlightsProps {
  searchStore: SearchStore;
  charWidth: number;
}

export function SearchHighlights({
  searchStore,
  charWidth,
}: SearchHighlightsProps) {
  const matches = useStoreSubscribeToField(searchStore.state, 'matches');
  const currentMatchIndex = useStoreSubscribeToField(
    searchStore.state,
    'currentMatchIndex',
  );

  if (matches.length === 0) {
    return null;
  }

  return (
    <>
      {matches.map((match, index) => (
        <SearchHighlight
          key={`${match.line}-${match.startColumn}-${match.endColumn}`}
          match={match}
          isCurrent={index === currentMatchIndex}
          charWidth={charWidth}
        />
      ))}
    </>
  );
}

interface SearchHighlightProps {
  match: SearchMatch;
  isCurrent: boolean;
  charWidth: number;
}

function SearchHighlight({
  match,
  isCurrent,
  charWidth,
}: SearchHighlightProps) {
  const left = LINE_PADDING_LEFT + match.startColumn * charWidth;
  const top = LINE_PADDING_TOP + match.line * CHAR_HEIGHT - 3;
  const width = (match.endColumn - match.startColumn) * charWidth;

  return (
    <div
      className={clsx(
        'absolute pointer-events-none',
        isCurrent ? 'bg-orange-500 opacity-60' : 'bg-yellow-500 opacity-40',
      )}
      style={{
        left: `${left}px`,
        top: `${top}px`,
        width: `${width}px`,
        height: `${CHAR_HEIGHT}px`,
      }}
    />
  );
}
