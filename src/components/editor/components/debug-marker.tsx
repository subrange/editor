import { useMemo, useRef, useLayoutEffect } from 'react';
import clsx from 'clsx';
import {
  useStoreSubscribe,
  useStoreSubscribeToField,
  useStoreSubscribeObservable,
} from '../../../hooks/use-store-subscribe.tsx';
import { EditorStore } from '../stores/editor.store.ts';
import { interpreterStore } from '../../debugger/interpreter-facade.store.ts';
import {
  LINE_PADDING_LEFT,
  LINE_PADDING_TOP,
  CHAR_HEIGHT,
} from '../constants.ts';
import { measureCharacterWidth } from '../../helpers.ts';

interface DebugMarkerProps {
  store: EditorStore;
}

export function DebugMarker({ store }: DebugMarkerProps) {
  const expandedPosition = useStoreSubscribe(interpreterStore.currentChar);
  const sourcePosition = useStoreSubscribeObservable(
    interpreterStore.currentSourceChar,
    false,
    null,
  );
  const cw = useMemo(() => measureCharacterWidth(), []);
  const debugMarkerRef = useRef<HTMLDivElement>(null);

  const isRunning = useStoreSubscribeToField(
    interpreterStore.state,
    'isRunning',
  );
  const isFinished = useStoreSubscribeToField(
    interpreterStore.state,
    'isStopped',
  );

  // Use source position for macro editor when available, expanded position for main editor
  const isMacroEditor = store.getId() === 'macro';
  const debugMarkerState =
    isMacroEditor && sourcePosition ? sourcePosition : expandedPosition;

  useLayoutEffect(() => {
    if (debugMarkerRef.current && (isRunning || !isFinished)) {
      debugMarkerRef.current.scrollIntoView({ block: 'center' });
    }
  });

  const stl = {
    left: `${LINE_PADDING_LEFT + debugMarkerState.column * cw}px`,
    top: `${LINE_PADDING_TOP + debugMarkerState.line * CHAR_HEIGHT - 3}px`,
    width: `${8}px`,
    height: `${CHAR_HEIGHT}px`,
  };

  const shouldShow = isMacroEditor
    ? sourcePosition && isRunning
    : isRunning || debugMarkerState.line !== 0 || debugMarkerState.column !== 0;

  return (
    shouldShow && (
      <div
        className={clsx(
          'absolute border border-green-500 pointer-events-none z-10',
          {},
        )}
        style={stl}
        ref={debugMarkerRef}
      />
    )
  );
}
