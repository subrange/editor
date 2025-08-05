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

// Maximum scrollable size to prevent browser issues (10 million pixels)
const MAX_SCROLL_SIZE = 10_000_000;

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
  const measuredSizes = useRef<Map<number, number>>(new Map());

  // Get estimated item size
  const estimatedItemSize = useMemo(() => estimateSize(), [estimateSize]);

  // Calculate if we need virtual scrolling based on total theoretical size
  const needsVirtualScroll = useMemo(() => {
    const theoreticalSize = paddingStart + (count * estimatedItemSize) + paddingEnd;
    return theoreticalSize > MAX_SCROLL_SIZE;
  }, [count, estimatedItemSize, paddingStart, paddingEnd]);

  // Calculate scroll ratio for mapping virtual positions
  const scrollRatio = useMemo(() => {
    if (!needsVirtualScroll) return 1;
    const theoreticalSize = paddingStart + (count * estimatedItemSize) + paddingEnd;
    return (MAX_SCROLL_SIZE - containerSize) / (theoreticalSize - containerSize);
  }, [needsVirtualScroll, count, estimatedItemSize, paddingStart, paddingEnd, containerSize]);

  // Get item size (from cache or estimate)
  const getItemSize = useCallback((index: number) => {
    return measuredSizes.current.get(index) ?? estimatedItemSize;
  }, [estimatedItemSize]);

  // Calculate total scrollable size
  const totalSize = useMemo(() => {
    if (!needsVirtualScroll) {
      // Normal calculation for smaller datasets
      return paddingStart + (count * estimatedItemSize) + paddingEnd;
    }
    // Cap at maximum for large datasets
    return MAX_SCROLL_SIZE;
  }, [needsVirtualScroll, count, estimatedItemSize, paddingStart, paddingEnd]);

  // Convert scroll position to actual item position
  const actualScrollOffset = useMemo(() => {
    if (!needsVirtualScroll) return scrollOffset;
    // Map from virtual scroll position to actual position
    return scrollOffset / scrollRatio;
  }, [scrollOffset, scrollRatio, needsVirtualScroll]);

  // Calculate visible range based on actual scroll position
  const visibleRange = useMemo(() => {
    if (!containerSize || count === 0) {
      return { start: 0, end: 0 };
    }

    // Calculate which items are visible based on actual offset
    const startIndex = Math.floor((actualScrollOffset - paddingStart) / estimatedItemSize);
    const endIndex = Math.ceil((actualScrollOffset + containerSize - paddingStart) / estimatedItemSize);

    // Apply overscan and bounds checking
    const overscanStart = Math.max(0, startIndex - overscan);
    const overscanEnd = Math.min(count - 1, endIndex + overscan);

    return { start: overscanStart, end: overscanEnd };
  }, [actualScrollOffset, containerSize, count, estimatedItemSize, paddingStart, overscan]);

  // Calculate positions only for visible items
  const visibleItemPositions = useMemo(() => {
    const positions: Array<{ index: number; start: number; size: number; end: number }> = [];
    
    if (visibleRange.start > visibleRange.end) {
      return positions;
    }

    // Calculate the starting position for the first visible item
    let offset = paddingStart + (visibleRange.start * estimatedItemSize);
    
    // For virtual scrolling, we need to map positions
    if (needsVirtualScroll) {
      offset = offset * scrollRatio;
    }

    // Calculate positions for visible items
    for (let i = visibleRange.start; i <= visibleRange.end; i++) {
      const size = getItemSize(i);
      
      positions.push({
        index: i,
        start: offset,
        size: size, // Keep original size
        end: offset + size,
      });
      offset += size;
    }

    return positions;
  }, [visibleRange, estimatedItemSize, paddingStart, getItemSize, needsVirtualScroll, scrollRatio]);

  // Get virtual items
  const getVirtualItems = useCallback(() => {
    return visibleItemPositions.map(pos => ({
      index: pos.index,
      key: getItemKey(pos.index),
      start: pos.start,
      size: pos.size,
      end: pos.end,
    }));
  }, [visibleItemPositions, getItemKey]);

  // Scroll to index
  const scrollToIndex = useCallback((index: number, options?: { align?: 'start' | 'center' | 'end' | 'auto' }) => {
    const scrollElement = scrollElementRef.current;
    if (!scrollElement || index < 0 || index >= count) return;

    // Calculate actual position of the item
    let actualPosition = paddingStart + (index * estimatedItemSize);
    const itemSize = getItemSize(index);
    
    // Calculate target offset based on alignment
    let targetOffset = actualPosition;
    const align = options?.align || 'auto';
    
    if (align === 'center') {
      targetOffset = actualPosition - (containerSize / 2) + (itemSize / 2);
    } else if (align === 'end') {
      targetOffset = actualPosition + itemSize - containerSize;
    } else if (align === 'auto') {
      const currentStart = actualScrollOffset;
      const currentEnd = actualScrollOffset + containerSize;
      
      if (actualPosition >= currentStart && actualPosition + itemSize <= currentEnd) {
        return; // Already visible
      }
      
      if (actualPosition < currentStart) {
        targetOffset = actualPosition;
      } else {
        targetOffset = actualPosition + itemSize - containerSize;
      }
    }

    // Map to virtual scroll position if needed
    if (needsVirtualScroll) {
      targetOffset = targetOffset * scrollRatio;
    }

    targetOffset = Math.max(0, Math.min(targetOffset, totalSize - containerSize));

    if (horizontal) {
      scrollElement.scrollLeft = targetOffset;
    } else {
      scrollElement.scrollTop = targetOffset;
    }
  }, [count, containerSize, horizontal, actualScrollOffset, totalSize, estimatedItemSize, paddingStart, getItemSize, needsVirtualScroll, scrollRatio]);

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

  // Memoize the return object to prevent unnecessary re-renders
  return useMemo(() => ({
    getVirtualItems,
    getTotalSize: () => totalSize,
    scrollToIndex,
  }), [getVirtualItems, totalSize, scrollToIndex]);
}