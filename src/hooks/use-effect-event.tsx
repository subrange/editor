import { useCallback, useInsertionEffect, useRef } from "react";

export function useEffectEvent<TArgs extends unknown[], TReturn>(
    fn: (...args: TArgs) => TReturn,
) {
    const ref = useRef<(...args: TArgs) => TReturn>(fn);
    useInsertionEffect(() => {
        ref.current = fn;
    }, [fn]);
    return useCallback((...args: TArgs) => {
        const f = ref.current;
        return f(...args);
    }, []);
}