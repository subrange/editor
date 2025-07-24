import {useLocalStorageState} from "../../hooks/use-local-storage-state.tsx";
import {useStoreSubscribeToField} from "../../hooks/use-store-subscribe.tsx";
import {interpreterStore} from "../debugger/interpreter.store.ts";
import {useLayoutEffect, useRef} from "react";
import clsx from "clsx";
import {ChevronDownIcon, ChevronUpDownIcon, ChevronUpIcon} from "@heroicons/react/16/solid";

export function Output() {
    const [collapsed, setCollapsed] = useLocalStorageState('outputCollapsed', true);

    const output = useStoreSubscribeToField(interpreterStore.state, "output");

    const outputContainer = useRef<HTMLDivElement>(null);

    // Scroll to the bottom when output changes
    useLayoutEffect(() => {
        setTimeout(() => {
            if (outputContainer.current) {
                outputContainer.current.scrollTop = outputContainer.current.scrollHeight;
            }
        }, 10);
    }, [output, outputContainer]);

    return <div className={clsx(
        "v  bg-zinc-900 transition-all", {
            "h-32 min-h-32": !collapsed,
            "h-8 min-h-8": collapsed,
        }
    )}>
        <button className={clsx(
            "h bg-zinc-900 text-zinc-500 text-xs font-bold p-2 h-8 min-h-8 border-t border-zinc-800 gap-2",
            "hover:bg-zinc-800 hover:text-zinc-400 transition-colors",
        )}
            onClick={() => setCollapsed(!collapsed)}
        >
            {
                collapsed
                    ? <ChevronUpIcon />
                    : <ChevronDownIcon />
            }
            Output
        </button>
        <div className="flex flex-col p-2 bg-zinc-950 grow-1 overflow-auto" ref={outputContainer}>
            <pre className="text-xs text-white overflow-x-auto whitespace-pre-wrap">
                {output}
            </pre>
        </div>
    </div>;
}