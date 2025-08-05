import { useRef, useState, useEffect, useCallback, useMemo } from 'react';

interface VirtualItem {
  index: number;
  key: string | number;
  start: number;
  size: number;
  end: number;
}

interface UseVirtualScrollOptions {
  count: number;
  estimateSize: (index?: number) => number;
  getScrollElement: () => HTMLElement | null;
  horizontal?: boolean;
  overscan?: number;
  paddingStart?: number;
  paddingEnd?: number;
  getItemKey?: (index: number) => string | number;
}

export function useVirtualScroll(options: UseVirtualScrollOptions) {
  const {
    count,
    estimateSize,
    getScrollElement,
    horizontal = false,
    overscan = 5,
    paddingStart = 0,
    paddingEnd = 0,
    getItemKey = (index) => index,
  } = options;

  const [scrollOffset, setScrollOffset] = useState(0);
  const [containerSize, setContainerSize] = useState(0);
  const scrollElementRef = useRef<HTMLElement | null>(null);

  // Get item size - it's constant for all items
  const itemSize = useMemo(() => estimateSize(), [estimateSize]);

  // Calculate total size
  const totalSize = useMemo(() => {
    return paddingStart + (count * itemSize) + paddingEnd;
  }, [count, itemSize, paddingStart, paddingEnd]);

  // Calculate visible range based on scroll position
  const visibleRange = useMemo(() => {
    if (!containerSize || count === 0) {
      return { start: 0, end: 0 };
    }

    // Direct calculation since all items have the same size
    const startIndex = Math.floor((scrollOffset - paddingStart) / itemSize);
    const endIndex = Math.ceil((scrollOffset + containerSize - paddingStart) / itemSize);

    // Apply overscan and bounds
    const overscanStart = Math.max(0, startIndex - overscan);
    const overscanEnd = Math.min(count - 1, endIndex + overscan);

    return { start: overscanStart, end: overscanEnd };
  }, [scrollOffset, containerSize, count, itemSize, paddingStart, overscan]);

  // Get virtual items with their positions
  const getVirtualItems = useCallback(() => {
    const items: VirtualItem[] = [];
    
    for (let i = visibleRange.start; i <= visibleRange.end; i++) {
      const start = paddingStart + (i * itemSize);
      
      items.push({
        index: i,
        key: getItemKey(i),
        start: start,
        size: itemSize,
        end: start + itemSize,
      });
    }

    return items;
  }, [visibleRange, paddingStart, itemSize, getItemKey]);

  // Scroll to index
  const scrollToIndex = useCallback((index: number, options?: { align?: 'start' | 'center' | 'end' | 'auto' }) => {
    const scrollElement = scrollElementRef.current;
    if (!scrollElement || index < 0 || index >= count) return;

    const itemStart = paddingStart + (index * itemSize);
    const itemEnd = itemStart + itemSize;
    const align = options?.align || 'auto';
    
    let targetOffset = itemStart;
    
    if (align === 'center') {
      targetOffset = itemStart - (containerSize / 2) + (itemSize / 2);
    } else if (align === 'end') {
      targetOffset = itemEnd - containerSize;
    } else if (align === 'auto') {
      const currentStart = scrollOffset;
      const currentEnd = scrollOffset + containerSize;
      
      if (itemStart >= currentStart && itemEnd <= currentEnd) {
        return; // Already visible
      }
      
      if (itemStart < currentStart) {
        targetOffset = itemStart;
      } else {
        targetOffset = itemEnd - containerSize;
      }
    }

    targetOffset = Math.max(0, Math.min(targetOffset, totalSize - containerSize));

    if (horizontal) {
      scrollElement.scrollLeft = targetOffset;
    } else {
      scrollElement.scrollTop = targetOffset;
    }
  }, [count, containerSize, horizontal, scrollOffset, totalSize, itemSize, paddingStart]);

  // Handle scroll event
  const handleScroll = useCallback(() => {
    const element = scrollElementRef.current;
    if (!element) return;

    const newOffset = horizontal ? element.scrollLeft : element.scrollTop;
    setScrollOffset(newOffset);
  }, [horizontal]);

  // Handle resize
  const handleResize = useCallback(() => {
    const element = scrollElementRef.current;
    if (!element) return;

    const newSize = horizontal ? element.clientWidth : element.clientHeight;
    setContainerSize(newSize);
  }, [horizontal]);

  // Set up event listeners
  useEffect(() => {
    const element = getScrollElement();
    if (!element) return;

    scrollElementRef.current = element;
    
    // Initial setup
    handleResize();
    handleScroll();

    // Event listeners
    element.addEventListener('scroll', handleScroll, { passive: true });
    window.addEventListener('resize', handleResize);

    const resizeObserver = new ResizeObserver(handleResize);
    resizeObserver.observe(element);

    return () => {
      element.removeEventListener('scroll', handleScroll);
      window.removeEventListener('resize', handleResize);
      resizeObserver.disconnect();
    };
  }, [getScrollElement, handleScroll, handleResize]);

  // Memoize the return object
  return useMemo(() => ({
    getVirtualItems,
    getTotalSize: () => {
      // Cap at a safe value to prevent browser issues
      // 8,999,999 is safely under 1e7 (10,000,000)
      const MAX_SAFE_CSS_SIZE = 8999999;
      return Math.min(totalSize, MAX_SAFE_CSS_SIZE);
    },
    scrollToIndex,
  }), [getVirtualItems, totalSize, scrollToIndex]);
}