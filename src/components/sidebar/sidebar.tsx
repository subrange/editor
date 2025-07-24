import {useStoreSubscribeToField} from "../../hooks/use-store-subscribe.tsx";
import {interpreterStore} from "../debugger/interpreter.store.ts";
import {useLayoutEffect, useRef} from "react";

function Output() {
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

    return <div className="v h-32 bg-zinc-800">
        <div className="v bg-zinc-700 text-white text-xs font-bold p-2 h-8 min-h-8">
            Output
        </div>
        <div className="flex flex-col p-2 bg-zinc-950 grow-1 overflow-auto" ref={outputContainer}>
            <pre className="text-xs text-white overflow-x-auto whitespace-pre-wrap">
                {output}
            </pre>
        </div>
    </div>;
}

export function Sidebar() {
    return (
        <div className="v w-80 min-w-80 h-screen bg-zinc-900">
            <div className="v grow-1">
            </div>
            <Output/>
        </div>
    )
}