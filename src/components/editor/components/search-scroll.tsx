import { useEffect, useRef } from "react";
import { useStoreSubscribe } from "../../../hooks/use-store-subscribe.tsx";
import { SearchStore } from "../stores/search.store.ts";
import { CHAR_HEIGHT, LINE_PADDING_LEFT, LINE_PADDING_TOP } from "../constants.ts";

interface SearchScrollProps {
    searchStore: SearchStore;
    charWidth: number;
}

export function SearchScroll({ searchStore, charWidth }: SearchScrollProps) {
    const searchState = useStoreSubscribe(searchStore.state);
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (scrollRef.current && searchState.currentMatchIndex >= 0) {
            scrollRef.current.scrollIntoView({ block: "center", inline: "center" });
        }
    }, [searchState.currentMatchIndex, searchState.scrollTrigger]);

    const currentMatch = searchStore.getCurrentMatch();
    if (!currentMatch) {
        return null;
    }

    const left = LINE_PADDING_LEFT + currentMatch.startColumn * charWidth;
    const top = LINE_PADDING_TOP + currentMatch.line * CHAR_HEIGHT - 3;

    return (
        <div
            ref={scrollRef}
            className="absolute pointer-events-none"
            style={{
                left: `${left}px`,
                top: `${top}px`,
                width: '1px',
                height: '1px',
            }}
        />
    );
}