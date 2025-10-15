import { useEffect, useMemo, useRef, useState } from 'react';
import { BehaviorSubject, map, Observable, Subscription } from 'rxjs';
import { distinctUntilChanged } from 'rxjs/operators';
import fastDeepEqual from 'fast-deep-equal';
import { useEffectEvent } from './use-effect-event.tsx';

export function useStoreSubscribe<T>(bs: BehaviorSubject<T>) {
  const [data, setData] = useState<T>(bs.getValue());
  const next = useRef<T>(data);

  const observer = useEffectEvent((newData: T) => {
    if (newData !== next.current) {
      next.current = newData;
      setData(newData);
    }
  });

  const subscription = useMemo<Subscription>(() => {
    return bs.subscribe(observer);
  }, [bs, observer]);

  useEffect(() => {
    return () => subscription.unsubscribe();
  }, [subscription]);

  return data;
}

export function useStoreSubscribeObservable<T>(
  bs: Observable<T>,
  performDeepEqual = false,
  initialState?: T,
) {
  const [data, setData] = useState<T | null>(initialState ?? null);

  const observer = useEffectEvent((newData: T) => {
    setData((oldData) => {
      if (!performDeepEqual) {
        return newData;
      }

      if (!fastDeepEqual(oldData, newData)) {
        return newData;
      }

      return oldData;
    });
  });

  const subscription = useMemo<Subscription>(() => {
    return bs.subscribe(observer);
  }, [bs, observer]);

  useEffect(() => {
    return () => subscription.unsubscribe();
  }, [subscription]);

  return data;
}

export function useStoreSubscribeToField<T extends object, K extends keyof T>(
  bs: BehaviorSubject<T>,
  storeItem: K,
  initialState?: T[K],
  deps: any[] = [], //eslint-disable-line @typescript-eslint/no-explicit-any
) {
  const [data, setData] = useState<T[K]>(
    initialState ?? bs.getValue()[storeItem],
  );

  useEffect(() => {
    const subscription = bs
      .pipe(
        map((state) => state[storeItem]),
        distinctUntilChanged(),
      )
      .subscribe((d) => {
        setData(d);
      });

    return () => {
      subscription.unsubscribe();
    };
  }, [bs, storeItem, ...deps]);

  return data;
}
