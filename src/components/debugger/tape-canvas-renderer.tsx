import { useEffect, useRef, useState, useCallback } from 'react';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe';
import { interpreterStore } from './interpreter-facade.store';
import { settingsStore } from '../../stores/settings.store';
import { tapeLabelsStore } from '../../stores/tape-labels.store';
import { XMarkIcon } from '@heroicons/react/24/solid';

interface TapeCanvasRendererProps {
  width: number;
  height: number;
  viewMode: 'normal' | 'compact' | 'lane';
  laneCount?: number;
}

// Lane colors from the DOM renderer
const LANE_COLORS = [
  { stroke: '#10b981', fill: 'rgba(16, 185, 129, 0.1)' }, // emerald
  { stroke: '#0ea5e9', fill: 'rgba(14, 165, 233, 0.1)' }, // sky
  { stroke: '#8b5cf6', fill: 'rgba(139, 92, 246, 0.1)' }, // violet
  { stroke: '#f43f5e', fill: 'rgba(244, 63, 94, 0.1)' },  // rose
  { stroke: '#f59e0b', fill: 'rgba(245, 158, 11, 0.1)' }, // amber
  { stroke: '#06b6d4', fill: 'rgba(6, 182, 212, 0.1)' },  // cyan
  { stroke: '#d946ef', fill: 'rgba(217, 70, 239, 0.1)' }, // fuchsia
  { stroke: '#84cc16', fill: 'rgba(132, 204, 22, 0.1)' }, // lime
  { stroke: '#f97316', fill: 'rgba(249, 115, 22, 0.1)' }, // orange
  { stroke: '#6366f1', fill: 'rgba(99, 102, 241, 0.1)' }, // indigo
];

export function TapeCanvasRenderer({ width, height, viewMode, laneCount = 1 }: TapeCanvasRendererProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationFrameRef = useRef<number>();
  const interpreterState = useStoreSubscribe(interpreterStore.state);
  const settings = useStoreSubscribe(settingsStore.settings);
  const labels = useStoreSubscribe(tapeLabelsStore.labels);
  
  const tape = interpreterState.tape;
  const pointer = interpreterState.pointer;
  const cellBits = tape instanceof Uint8Array ? 8 : tape instanceof Uint16Array ? 16 : 32;
  
  // Scroll state
  const [scrollX, setScrollX] = useState(0);
  const [scrollY, setScrollY] = useState(0);
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const [hoveredColumn, setHoveredColumn] = useState<number | null>(null);
  const [hoveredLane, setHoveredLane] = useState<number | null>(null);
  
  // Scroll bar dragging state
  const [isDraggingScrollBar, setIsDraggingScrollBar] = useState(false);
  const [isDraggingVerticalScrollBar, setIsDraggingVerticalScrollBar] = useState(false);
  const [scrollBarDragStart, setScrollBarDragStart] = useState<{ mouseX: number; scrollX: number } | null>(null);
  const [verticalScrollBarDragStart, setVerticalScrollBarDragStart] = useState<{ mouseY: number; scrollY: number } | null>(null);
  const [isCanvasHovered, setIsCanvasHovered] = useState(false);
  const [isScrollBarHovered, setIsScrollBarHovered] = useState(false);
  const [isVerticalScrollBarHovered, setIsVerticalScrollBarHovered] = useState(false);
  
  // Context menu state
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; type: 'lane' | 'column' | 'cell'; index: number } | null>(null);
  
  // Animation state for smooth transitions
  const animationStateRef = useRef<Map<number, { 
    currentOpacity: number; 
    targetOpacity: number;
    lastUpdate: number;
  }>>(new Map());
  
  // Cell dimensions based on view mode
  const dimensions = viewMode === 'compact' ? {
    cellWidth: 60,
    cellHeight: 24,
    cellGap: 2,
    padding: 12,
    fontSize: { index: 8, value: 12, binary: 8, ascii: 10 }
  } : viewMode === 'lane' && laneCount > 1 ? {
    cellWidth: 80,
    cellHeight: 24,
    cellGap: 8,
    padding: 48,
    fontSize: { index: 9, value: 12, binary: 0, ascii: 0 }
  } : {
    cellWidth: 100,
    cellHeight: 120,
    cellGap: 8,
    padding: 24,
    fontSize: { index: 12, value: 24, binary: 10, ascii: 12 }
  };
  
  const CELL_WIDTH = dimensions.cellWidth;
  const CELL_HEIGHT = dimensions.cellHeight;
  const CELL_GAP = dimensions.cellGap;
  const PADDING = dimensions.padding;
  
  // Calculate total virtual width
  const totalWidth = PADDING + (tape.length * (CELL_WIDTH + CELL_GAP)) - CELL_GAP + PADDING;
  
  // Virtual scrolling calculations
  const visibleStartX = scrollX - CELL_WIDTH; // Add buffer
  const visibleEndX = scrollX + width + CELL_WIDTH;
  const firstVisibleIndex = Math.max(0, Math.floor((visibleStartX - PADDING) / (CELL_WIDTH + CELL_GAP)));
  const lastVisibleIndex = Math.min(tape.length - 1, Math.ceil((visibleEndX - PADDING) / (CELL_WIDTH + CELL_GAP)));
  
  // Easing function (ease-out-cubic)
  const easeOutCubic = (t: number): number => {
    return 1 - Math.pow(1 - t, 3);
  };
  
  // Get animated opacity for a cell
  const getAnimatedOpacity = useCallback((index: number, isDimmed: boolean): number => {
    const targetOpacity = isDimmed ? 0.3 : 1;
    const now = Date.now();
    const animationDuration = 200; // ms
    
    let state = animationStateRef.current.get(index);
    if (!state) {
      state = { currentOpacity: targetOpacity, targetOpacity, lastUpdate: now };
      animationStateRef.current.set(index, state);
      return targetOpacity;
    }
    
    if (state.targetOpacity !== targetOpacity) {
      state.targetOpacity = targetOpacity;
      state.lastUpdate = now;
    }
    
    const elapsed = now - state.lastUpdate;
    if (elapsed >= animationDuration) {
      state.currentOpacity = targetOpacity;
    } else {
      const progress = elapsed / animationDuration;
      const easedProgress = easeOutCubic(progress);
      const startOpacity = state.currentOpacity;
      state.currentOpacity = startOpacity + (targetOpacity - startOpacity) * easedProgress;
    }
    
    return state.currentOpacity;
  }, []);
  
  // Draw function
  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext('2d');
    if (!canvas || !ctx) return;
    
    // Special handling for lane view
    if (viewMode === 'lane' && laneCount > 1) {
      drawLaneView(ctx);
      return;
    }
    
    // Set canvas size with device pixel ratio for sharp rendering
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.scale(dpr, dpr);
    
    // Clear canvas
    ctx.fillStyle = '#09090b'; // zinc-950
    ctx.fillRect(0, 0, width, height);
    
    // Draw visible cells without using translate
    for (let i = firstVisibleIndex; i <= lastVisibleIndex; i++) {
      // Calculate virtual position and then subtract scrollX to get screen position
      const virtualX = PADDING + i * (CELL_WIDTH + CELL_GAP);
      const x = virtualX - scrollX;
      const y = height / 2 - CELL_HEIGHT / 2;
      const value = tape[i];
      const isPointer = i === pointer;
      const isHovered = i === hoveredIndex;
      const isDimmed = hoveredIndex !== null && !isHovered;
      
      // Get animated opacity
      const opacity = getAnimatedOpacity(i, isDimmed);
      
      // Cell background with animated opacity
      if (isPointer) {
        ctx.fillStyle = `rgba(234, 179, 8, ${0.1 * opacity})`; // yellow-500
        ctx.strokeStyle = `rgba(234, 179, 8, ${0.5 * opacity})`;
        ctx.lineWidth = 2;
      } else if (value !== 0) {
        ctx.fillStyle = `rgba(59, 130, 246, ${0.1 * opacity})`; // blue-500
        ctx.strokeStyle = `rgba(59, 130, 246, ${0.3 * opacity})`;
        ctx.lineWidth = 1;
      } else {
        ctx.fillStyle = `rgba(63, 63, 70, ${0.5 * opacity})`; // zinc-700
        ctx.strokeStyle = `rgba(63, 63, 70, ${0.4 * opacity})`;
        ctx.lineWidth = 1;
      }
      
      // Draw cell
      ctx.beginPath();
      ctx.roundRect(x, y, CELL_WIDTH, CELL_HEIGHT, 4);
      ctx.fill();
      ctx.stroke();
      
      // Draw hover effect
      if (isHovered && !isPointer) {
        ctx.strokeStyle = 'rgba(161, 161, 170, 0.6)'; // zinc-400 with reduced opacity
        ctx.lineWidth = 2;
        ctx.stroke();
      }
      
      // Draw cell index or label
      if (dimensions.fontSize.index > 0) {
        const hasLabel = labels.cells[i] !== undefined;
        ctx.fillStyle = isPointer ? 
          `rgba(250, 204, 21, ${opacity})` : 
          hasLabel ? `rgba(161, 161, 170, ${opacity})` : // zinc-400 for labels
          `rgba(113, 113, 122, ${opacity})`; // zinc-500 for indices
        ctx.font = `${dimensions.fontSize.index}px monospace`;
        ctx.textAlign = 'center';
        const cellLabel = labels.cells[i] || `#${i}`;
        ctx.fillText(cellLabel, x + CELL_WIDTH / 2 + (viewMode === 'compact' ? 0 : 0), y + (viewMode === 'compact' ? 6 : 20));
      }
      
      // Draw value
      ctx.fillStyle = isPointer ? 
        `rgba(253, 224, 71, ${opacity})` : 
        value !== 0 ? `rgba(147, 197, 253, ${opacity})` : 
        `rgba(161, 161, 170, ${opacity})`; // yellow-300 : blue-300 : zinc-400
      ctx.font = `bold ${dimensions.fontSize.value}px monospace`;
      ctx.textAlign = 'center';
      ctx.fillText(value.toString(), x + CELL_WIDTH / 2, y + CELL_HEIGHT / 2 + (viewMode === 'compact' ? 6 : 8));
      
      // Draw binary representation for small values (not in compact mode)
      if (cellBits === 8 && dimensions.fontSize.binary > 0) {
        const binary = value.toString(2).padStart(8, '0');
        ctx.fillStyle = isPointer ? 
          `rgba(250, 204, 21, ${0.7 * opacity})` : 
          value !== 0 ? `rgba(147, 197, 253, ${0.7 * opacity})` : 
          `rgba(82, 82, 91, ${opacity})`; // zinc-600
        ctx.font = `${dimensions.fontSize.binary}px monospace`;
        ctx.fillText(binary, x + CELL_WIDTH / 2, y + CELL_HEIGHT - 20);
      }
      
      // Draw ASCII for printable 8-bit values (not in compact mode)
      if (cellBits === 8 && value >= 32 && value <= 126 && dimensions.fontSize.ascii > 0) {
        ctx.fillStyle = isPointer ? 
          `rgba(250, 204, 21, ${opacity})` : 
          `rgba(161, 161, 170, ${opacity})`;
        ctx.font = `${dimensions.fontSize.ascii}px monospace`;
        ctx.fillText(`'${String.fromCharCode(value)}'`, x + CELL_WIDTH / 2, y + CELL_HEIGHT - 8);
      }
      
      // Draw pointer indicator
      if (isPointer && viewMode === 'normal') {
        ctx.fillStyle = `rgba(234, 179, 8, ${opacity})`;
        ctx.beginPath();
        ctx.moveTo(x + CELL_WIDTH / 2 - 6, y + CELL_HEIGHT + 5);
        ctx.lineTo(x + CELL_WIDTH / 2 + 6, y + CELL_HEIGHT + 5);
        ctx.lineTo(x + CELL_WIDTH / 2, y + CELL_HEIGHT + 11);
        ctx.closePath();
        ctx.fill();
      }
    }
    
    // Draw scroll indicator at bottom
    if (totalWidth > width) {
      const scrollBarHeight = 8; // Made taller for easier dragging
      const scrollBarY = height - scrollBarHeight - 10;
      const scrollBarWidth = Math.max(50, (width / totalWidth) * width);
      const scrollBarX = (scrollX / (totalWidth - width)) * (width - scrollBarWidth);
      
      // Calculate opacity based on hover state
      const opacity = isDraggingScrollBar ? 0.8 : 
                     isScrollBarHovered ? 0.6 : 
                     isCanvasHovered ? 0.2 : 0;
      
      if (opacity > 0) {
        // Scroll track
        ctx.fillStyle = `rgba(39, 39, 42, ${opacity})`; // zinc-800 with opacity
        ctx.fillRect(0, scrollBarY, width, scrollBarHeight);
        
        // Scroll thumb
        const thumbColor = isDraggingScrollBar ? '113, 113, 122' : '82, 82, 91'; // zinc-500 : zinc-600
        ctx.fillStyle = `rgba(${thumbColor}, ${opacity})`;
        ctx.fillRect(scrollBarX, scrollBarY, scrollBarWidth, scrollBarHeight);
      }
      
      // Store scroll bar dimensions for hit testing
      if (!canvasRef.current) return;
      (canvasRef.current as any)._scrollBarBounds = {
        x: scrollBarX,
        y: scrollBarY,
        width: scrollBarWidth,
        height: scrollBarHeight
      };
    } else {
      // Clear scroll bar bounds if no scroll bar
      if (canvasRef.current) {
        (canvasRef.current as any)._scrollBarBounds = null;
      }
    }
    
    // Draw stats in corner
    // ctx.fillStyle = '#a1a1aa'; // zinc-400
    // ctx.font = '11px monospace';
    // ctx.textAlign = 'left';
    // ctx.fillText(`Memory: ${tape.length.toLocaleString()} cells | Pointer: ${pointer} | Value: ${tape[pointer]}`, 10, 20);
  }, [width, height, scrollX, tape, pointer, cellBits, firstVisibleIndex, lastVisibleIndex, hoveredIndex, totalWidth, viewMode, dimensions, getAnimatedOpacity, isDraggingScrollBar, isScrollBarHovered, isCanvasHovered]);
  
  // Draw lane view
  const drawLaneView = useCallback((ctx: CanvasRenderingContext2D) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    // Set canvas size with device pixel ratio
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.scale(dpr, dpr);
    
    // Clear canvas
    ctx.fillStyle = '#09090b';
    ctx.fillRect(0, 0, width, height);
    
    // Calculate columns
    const columnsCount = Math.ceil(tape.length / laneCount);
    const columnWidth = CELL_WIDTH + CELL_GAP;
    const laneHeight = CELL_HEIGHT + 4; // 4px gap between lanes
    const COLUMN_OFFSET = 8; // Add small offset after lane numbers panel
    const HEADER_HEIGHT = 25;
    const totalLanesHeight = laneCount * laneHeight;
    const viewportHeight = height - HEADER_HEIGHT - 20; // 20px for bottom padding
    
    // Visible columns and lanes
    const firstColumn = Math.max(0, Math.floor((scrollX - PADDING - COLUMN_OFFSET) / columnWidth));
    const lastColumn = Math.min(columnsCount - 1, Math.ceil((scrollX + width - PADDING - COLUMN_OFFSET) / columnWidth));
    const firstLane = Math.max(0, Math.floor(scrollY / laneHeight));
    const lastLane = Math.min(laneCount - 1, Math.ceil((scrollY + viewportHeight) / laneHeight));
    
    // Set up clipping region for scrollable content
    ctx.save();
    ctx.beginPath();
    ctx.rect(PADDING, 0, width - PADDING, height);
    ctx.clip();

    // Draw column headers (scrollable)
    ctx.fillStyle = '#18181b';
    const headerWidth = Math.min(columnsCount * columnWidth, 100000000); // Prevent overflow
    const headerX = PADDING - scrollX;
    ctx.fillRect(headerX, 0, headerWidth, 25);
    
    for (let col = firstColumn; col <= lastColumn && col < columnsCount; col++) {
      const virtualX = PADDING + COLUMN_OFFSET + col * columnWidth + CELL_WIDTH / 2;
      const x = virtualX - scrollX;
      ctx.fillStyle = '#71717a';
      ctx.font = '10px monospace';
      ctx.textAlign = 'center';
      const columnLabel = labels.columns[col] || col.toString();
      ctx.fillText(columnLabel, x, 18);
    }
    
    // Draw cells
    for (let col = firstColumn; col <= lastColumn; col++) {
      for (let lane = firstLane; lane <= lastLane; lane++) {
        const index = col * laneCount + lane;
        if (index >= tape.length) continue;
        
        const virtualX = PADDING + COLUMN_OFFSET + col * columnWidth;
        const x = virtualX - scrollX;
        const virtualY = HEADER_HEIGHT + 5 + lane * laneHeight;
        const y = virtualY - scrollY;
        const value = tape[index];
        const isPointer = index === pointer;
        const isHovered = index === hoveredIndex;
        const laneColor = LANE_COLORS[lane % LANE_COLORS.length];
        const isDimmed = hoveredIndex !== null && hoveredColumn !== null && hoveredLane !== null && 
                        (col !== hoveredColumn && lane !== hoveredLane);
        
        // Get animated opacity
        const opacity = getAnimatedOpacity(index, isDimmed);
        
        // Cell background with animated opacity
        const hasLabel = labels.cells[index] !== undefined;
        if (isPointer) {
          ctx.fillStyle = `rgba(234, 179, 8, ${0.2 * opacity})`;
          ctx.strokeStyle = `rgba(234, 179, 8, ${0.5 * opacity})`;
          ctx.lineWidth = 2;
        } else {
          // Parse the lane color and apply opacity
          const rgbaMatch = laneColor.fill.match(/rgba?\((\d+),\s*(\d+),\s*(\d+),?\s*([\d.]+)?\)/);
          if (rgbaMatch) {
            const [, r, g, b] = rgbaMatch;
            // Make fillstype brighter for labeled cells
            const opacityFactor = hasLabel ? 0.2 : 0.1;
            ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${opacityFactor * opacity})`;
          } else {
            ctx.fillStyle = laneColor.fill;
          }
          
          const strokeMatch = laneColor.stroke.match(/#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})/i);
          if (strokeMatch) {
            const [, r, g, b] = strokeMatch;
            const rDec = parseInt(r, 16);
            const gDec = parseInt(g, 16);
            const bDec = parseInt(b, 16);
            // Make border brighter for labeled cells
            const strokeOpacity = hasLabel ? 0.8 : 0.4;
            ctx.strokeStyle = `rgba(${rDec}, ${gDec}, ${bDec}, ${strokeOpacity * opacity})`;
          } else {
            ctx.strokeStyle = laneColor.stroke;
          }
          ctx.lineWidth = hasLabel ? 1.5 : 1; // Slightly thicker border for labeled cells
        }
        
        ctx.beginPath();
        ctx.roundRect(x, y, CELL_WIDTH, CELL_HEIGHT, 2);
        ctx.fill();
        ctx.stroke();
        
        // Draw hover effect
        if (isHovered && !isPointer) {
          ctx.strokeStyle = 'rgba(228, 228, 231, 0.5)'; // zinc-200 with reduced opacity
          ctx.lineWidth = 2;
          ctx.stroke();
        }
        
        // Cell content with animated opacity - show label or index
        ctx.fillStyle = hasLabel ?
          `rgba(161, 161, 170, ${opacity})` : // zinc-400 for labels
          `rgba(113, 113, 122, ${opacity})`; // zinc-500 for indices
        ctx.font = '9px monospace';
        ctx.textAlign = 'left';
        const cellLabel = labels.cells[index] || index.toString();
        ctx.fillText(cellLabel, x + 3, y + CELL_HEIGHT / 2 - 2);
        
        ctx.fillStyle = isPointer ? 
          `rgba(253, 224, 71, ${opacity})` : 
          value !== 0 ? `rgba(147, 197, 253, ${opacity})` : 
          `rgba(161, 161, 170, ${0.5 * opacity})`;
        ctx.font = 'bold 12px monospace';
        ctx.textAlign = 'right';
        ctx.fillText(value.toString(), x + CELL_WIDTH - 3, y + CELL_HEIGHT / 2 + 4);
      }
    }
    
    ctx.restore();
    
    // Draw fixed lane headers (not scrolled)
    ctx.fillStyle = '#18181b'; // zinc-900
    ctx.fillRect(0, 0, PADDING, height);
    
    // Draw lane numbers
    for (let lane = firstLane; lane <= lastLane; lane++) {
      const virtualY = HEADER_HEIGHT + 5 + lane * laneHeight;
      const y = virtualY - scrollY;
      
      // Subtle background for lane rows
      if (lane % 2 === 0) {
        ctx.fillStyle = 'rgba(63, 63, 70, 0.1)'; // zinc-700/10
        ctx.fillRect(0, y, PADDING, laneHeight - 4);
      }
      
      // Lane number or label
      ctx.fillStyle = '#a1a1aa';
      ctx.font = '11px monospace';
      ctx.textAlign = 'center';
      const laneLabel = labels.lanes[lane] || lane.toString();
      ctx.fillText(laneLabel, PADDING / 2, y + laneHeight / 2 + 3);
    }
    
    // Draw vertical separator
    ctx.strokeStyle = '#3f3f46'; // zinc-700
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(PADDING - 0.5, 0);
    ctx.lineTo(PADDING - 0.5, height);
    ctx.stroke();
    
    // Draw horizontal separator under headers
    ctx.beginPath();
    ctx.moveTo(0, HEADER_HEIGHT);
    ctx.lineTo(width, HEADER_HEIGHT);
    ctx.stroke();
    
    // Draw corner cell LAST so it's always on top
    ctx.fillStyle = '#09090b'; // zinc-950
    ctx.fillRect(0, 0, PADDING, HEADER_HEIGHT);
    ctx.strokeStyle = '#3f3f46'; // zinc-700
    ctx.lineWidth = 1;
    ctx.strokeRect(0, 0, PADDING, HEADER_HEIGHT);
    ctx.fillStyle = '#71717a';
    ctx.font = '9px monospace';
    ctx.textAlign = 'center';
    ctx.fillText('L/W', PADDING / 2, 16);
    
    // Draw scroll indicator
    if (columnsCount * columnWidth > width) {
      const scrollBarHeight = 8; // Made taller for easier dragging
      const scrollBarY = height - scrollBarHeight - 10;
      const totalVirtualWidth = PADDING + COLUMN_OFFSET + columnsCount * columnWidth + PADDING;
      const scrollBarWidth = Math.max(50, (width / totalVirtualWidth) * width);
      const scrollBarX = (scrollX / (totalVirtualWidth - width)) * (width - scrollBarWidth);
      
      // Calculate opacity based on hover state
      const opacity = isDraggingScrollBar ? 0.8 : 
                     isScrollBarHovered ? 0.6 : 
                     isCanvasHovered ? 0.2 : 0;
      
      if (opacity > 0) {
        // Scroll track
        ctx.fillStyle = `rgba(39, 39, 42, ${opacity})`; // zinc-800 with opacity
        ctx.fillRect(0, scrollBarY, width, scrollBarHeight);
        
        // Scroll thumb
        const thumbColor = isDraggingScrollBar ? '113, 113, 122' : '82, 82, 91'; // zinc-500 : zinc-600
        ctx.fillStyle = `rgba(${thumbColor}, ${opacity})`;
        ctx.fillRect(scrollBarX, scrollBarY, scrollBarWidth, scrollBarHeight);
      }
      
      // Store scroll bar dimensions for hit testing
      if (!canvasRef.current) return;
      (canvasRef.current as any)._scrollBarBounds = {
        x: scrollBarX,
        y: scrollBarY,
        width: scrollBarWidth,
        height: scrollBarHeight
      };
    } else {
      // Clear scroll bar bounds if no scroll bar
      if (canvasRef.current) {
        (canvasRef.current as any)._scrollBarBounds = null;
      }
    }
    
    // Draw vertical scroll indicator (for lanes)
    if (totalLanesHeight > viewportHeight) {
      const vScrollBarWidth = 8;
      const vScrollBarX = width - vScrollBarWidth - 10;
      const vScrollBarHeight = Math.max(50, (viewportHeight / totalLanesHeight) * viewportHeight);
      const vScrollBarY = HEADER_HEIGHT + (scrollY / (totalLanesHeight - viewportHeight)) * (viewportHeight - vScrollBarHeight);
      
      // Calculate opacity based on hover state
      const opacity = isDraggingVerticalScrollBar ? 0.8 : 
                     isVerticalScrollBarHovered ? 0.6 : 
                     isCanvasHovered ? 0.2 : 0;
      
      if (opacity > 0) {
        // Scroll track
        ctx.fillStyle = `rgba(39, 39, 42, ${opacity})`;
        ctx.fillRect(vScrollBarX, HEADER_HEIGHT, vScrollBarWidth, viewportHeight);
        
        // Scroll thumb
        const thumbColor = isDraggingVerticalScrollBar ? '113, 113, 122' : '82, 82, 91';
        ctx.fillStyle = `rgba(${thumbColor}, ${opacity})`;
        ctx.fillRect(vScrollBarX, vScrollBarY, vScrollBarWidth, vScrollBarHeight);
      }
      
      // Store vertical scroll bar dimensions for hit testing
      if (!canvasRef.current) return;
      (canvasRef.current as any)._verticalScrollBarBounds = {
        x: vScrollBarX,
        y: vScrollBarY,
        width: vScrollBarWidth,
        height: vScrollBarHeight
      };
    } else {
      // Clear vertical scroll bar bounds if no scroll bar
      if (canvasRef.current) {
        (canvasRef.current as any)._verticalScrollBarBounds = null;
      }
    }
  }, [tape, pointer, laneCount, scrollX, scrollY, width, height, PADDING, CELL_WIDTH, CELL_HEIGHT, CELL_GAP, hoveredIndex, hoveredColumn, hoveredLane, getAnimatedOpacity, isDraggingScrollBar, isScrollBarHovered, isCanvasHovered, isDraggingVerticalScrollBar, isVerticalScrollBarHovered, labels]);
  
  // Animation loop
  useEffect(() => {
    const animate = () => {
      draw();
      
      // Clean up old animation states periodically
      const now = Date.now();
      if (animationStateRef.current.size > 1000) {
        const toDelete: number[] = [];
        animationStateRef.current.forEach((state, index) => {
          if (now - state.lastUpdate > 1000) { // Remove states older than 1 second
            toDelete.push(index);
          }
        });
        toDelete.forEach(index => animationStateRef.current.delete(index));
      }
      
      animationFrameRef.current = requestAnimationFrame(animate);
    };
    animate();
    
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [draw]);
  
  // Track previous pointer position to detect wrapping
  const prevPointerRef = useRef(pointer);
  
  // Auto-scroll to pointer
  useEffect(() => {
    const prevPointer = prevPointerRef.current;
    prevPointerRef.current = pointer;
    
    // Detect if pointer wrapped (jumped from near start to near end or vice versa)
    const isWrap = Math.abs(pointer - prevPointer) > tape.length * 0.9;
    
    let pointerX: number;
    let effectiveWidth: number;
    
    if (viewMode === 'lane' && laneCount > 1) {
      // In lane view, calculate based on columns
      const pointerColumn = Math.floor(pointer / laneCount);
      const COLUMN_OFFSET = 8; // Same offset as in drawLaneView
      pointerX = PADDING + COLUMN_OFFSET + pointerColumn * (CELL_WIDTH + CELL_GAP);
      const columnsCount = Math.ceil(tape.length / laneCount);
      effectiveWidth = PADDING + COLUMN_OFFSET + columnsCount * (CELL_WIDTH + CELL_GAP) + PADDING;
    } else {
      pointerX = PADDING + pointer * (CELL_WIDTH + CELL_GAP);
      effectiveWidth = totalWidth;
    }
    
    const pointerInView = pointerX >= scrollX && pointerX + CELL_WIDTH <= scrollX + width;
    
    if (!pointerInView || isWrap) {
      // Center the pointer in view
      const targetScroll = pointerX - width / 2 + CELL_WIDTH / 2;
      const newScroll = Math.max(0, Math.min(targetScroll, effectiveWidth - width));
      
      // Use requestAnimationFrame to ensure clean rendering when wrapping
      if (isWrap) {
        // Clear animation states to prevent visual artifacts
        animationStateRef.current.clear();
        requestAnimationFrame(() => {
          setScrollX(newScroll);
        });
      } else {
        setScrollX(newScroll);
      }
    }
  }, [pointer, width, totalWidth, viewMode, laneCount, tape.length, CELL_WIDTH, CELL_GAP, PADDING]);
  
  // Handle mouse wheel scrolling
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    
    if (viewMode === 'lane' && laneCount > 1) {
      const deltaX = e.deltaX;
      const deltaY = e.deltaY;
      
      // Handle horizontal scrolling
      if (Math.abs(deltaX) > Math.abs(deltaY) || e.shiftKey) {
        const delta = e.shiftKey ? deltaY : deltaX;
        const newScrollX = scrollX + delta;
        const columnsCount = Math.ceil(tape.length / laneCount);
        const COLUMN_OFFSET = 8;
        const laneViewWidth = PADDING + COLUMN_OFFSET + columnsCount * (CELL_WIDTH + CELL_GAP) + PADDING;
        const maxScrollX = Math.max(0, laneViewWidth - width);
        setScrollX(Math.max(0, Math.min(newScrollX, maxScrollX)));
      } else {
        // Handle vertical scrolling
        const newScrollY = scrollY + deltaY;
        const HEADER_HEIGHT = 25;
        const totalLanesHeight = laneCount * (CELL_HEIGHT + 4);
        const viewportHeight = height - HEADER_HEIGHT - 20;
        const maxScrollY = Math.max(0, totalLanesHeight - viewportHeight);
        setScrollY(Math.max(0, Math.min(newScrollY, maxScrollY)));
      }
    } else {
      // Normal/compact view - only horizontal scrolling
      const delta = e.deltaX || e.deltaY;
      const newScroll = scrollX + delta;
      const maxScroll = Math.max(0, totalWidth - width);
      setScrollX(Math.max(0, Math.min(newScroll, maxScroll)));
    }
  }, [scrollX, scrollY, totalWidth, width, height, viewMode, laneCount, tape.length, CELL_WIDTH, CELL_HEIGHT, CELL_GAP, PADDING]);
  
  // Handle mouse move for hover
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;
    
    // Check if hovering over horizontal scroll bar
    const scrollBarBounds = canvasRef.current ? (canvasRef.current as any)._scrollBarBounds : null;
    if (scrollBarBounds) {
      const { x, y, width, height } = scrollBarBounds;
      const isOverScrollBar = mouseX >= x && mouseX <= x + width && mouseY >= y && mouseY <= y + height;
      setIsScrollBarHovered(isOverScrollBar);
    } else {
      setIsScrollBarHovered(false);
    }
    
    // Check if hovering over vertical scroll bar
    const verticalScrollBarBounds = canvasRef.current ? (canvasRef.current as any)._verticalScrollBarBounds : null;
    if (verticalScrollBarBounds) {
      const { x, y, width, height } = verticalScrollBarBounds;
      const isOverVerticalScrollBar = mouseX >= x && mouseX <= x + width && mouseY >= y && mouseY <= y + height;
      setIsVerticalScrollBarHovered(isOverVerticalScrollBar);
    } else {
      setIsVerticalScrollBarHovered(false);
    }
    
    if (viewMode === 'lane' && laneCount > 1) {
      // Lane view hover detection
      if (mouseX < PADDING || mouseY < 25) {
        setHoveredIndex(null);
        setHoveredColumn(null);
        setHoveredLane(null);
        return;
      }
      
      const COLUMN_OFFSET = 8; // Same offset as in drawLaneView
      const scrolledX = mouseX - PADDING - COLUMN_OFFSET + scrollX;
      const HEADER_HEIGHT = 25;
      const relativeY = mouseY - HEADER_HEIGHT - 5 + scrollY;
      
      // Simple calculation - each cell owns its space plus gap
      const col = Math.floor(scrolledX / (CELL_WIDTH + CELL_GAP));
      const lane = Math.floor(relativeY / (CELL_HEIGHT + 4));
      
      if (col >= 0 && lane >= 0 && lane < laneCount && relativeY >= 0) {
        const columnsCount = Math.ceil(tape.length / laneCount);
        if (col < columnsCount) {
          const index = col * laneCount + lane;
          if (index < tape.length) {
            setHoveredIndex(index);
            setHoveredColumn(col);
            setHoveredLane(lane);
            return;
          }
        }
      }
      setHoveredIndex(null);
      setHoveredColumn(null);
      setHoveredLane(null);
    } else {
      // Normal/compact view hover detection
      const x = mouseX + scrollX;
      const cellX = x - PADDING;
      
      if (cellX >= 0) {
        // Calculate which cell we're hovering over by dividing the position
        // Each cell "owns" the space from its start to the start of the next cell
        const totalCellWidth = CELL_WIDTH + CELL_GAP;
        const index = Math.floor(cellX / totalCellWidth);
        
        if (index >= 0 && index < tape.length) {
          setHoveredIndex(index);
          return;
        }
      }
    }
    
    setHoveredIndex(null);
    setHoveredColumn(null);
    setHoveredLane(null);
  }, [scrollX, scrollY, tape.length, viewMode, laneCount, CELL_WIDTH, CELL_HEIGHT, CELL_GAP, PADDING]);
  
  const handleMouseLeave = useCallback(() => {
    setHoveredIndex(null);
    setHoveredColumn(null);
    setHoveredLane(null);
    setIsCanvasHovered(false);
    setIsScrollBarHovered(false);
    setIsVerticalScrollBarHovered(false);
  }, []);
  
  const handleMouseEnter = useCallback(() => {
    setIsCanvasHovered(true);
  }, []);
  
  // Handle mouse down for scroll bar dragging
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect || !canvasRef.current) return;
    
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;
    
    // Check if mouse is on horizontal scroll bar
    const scrollBarBounds = (canvasRef.current as any)._scrollBarBounds;
    if (scrollBarBounds) {
      const { x, y, width, height } = scrollBarBounds;
      if (mouseX >= x && mouseX <= x + width && mouseY >= y && mouseY <= y + height) {
        setIsDraggingScrollBar(true);
        setScrollBarDragStart({ mouseX: e.clientX, scrollX });
        e.preventDefault();
        return;
      }
    }
    
    // Check if mouse is on vertical scroll bar
    const verticalScrollBarBounds = (canvasRef.current as any)._verticalScrollBarBounds;
    if (verticalScrollBarBounds) {
      const { x, y, width, height } = verticalScrollBarBounds;
      if (mouseX >= x && mouseX <= x + width && mouseY >= y && mouseY <= y + height) {
        setIsDraggingVerticalScrollBar(true);
        setVerticalScrollBarDragStart({ mouseY: e.clientY, scrollY });
        e.preventDefault();
      }
    }
  }, [scrollX, scrollY]);
  
  // Handle right click for context menu
  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;
    
    // Lane view specific headers
    if (viewMode === 'lane' && laneCount > 1) {
      // Check if clicked on lane header
      if (mouseX < PADDING && mouseY > 25) {
        const HEADER_HEIGHT = 25;
        const relativeY = mouseY - HEADER_HEIGHT - 5 + scrollY;
        const lane = Math.floor(relativeY / (CELL_HEIGHT + 4));
        if (lane >= 0 && lane < laneCount) {
          setContextMenu({
            x: e.clientX,
            y: e.clientY,
            type: 'lane',
            index: lane
          });
          return;
        }
      }
      
      // Check if clicked on column header
      if (mouseY < 25 && mouseX > PADDING) {
        const COLUMN_OFFSET = 8;
        const scrolledX = mouseX - PADDING - COLUMN_OFFSET + scrollX;
        const col = Math.floor(scrolledX / (CELL_WIDTH + CELL_GAP));
        const columnsCount = Math.ceil(tape.length / laneCount);
        if (col >= 0 && col < columnsCount) {
          setContextMenu({
            x: e.clientX,
            y: e.clientY,
            type: 'column',
            index: col
          });
          return;
        }
      }
    }
    
    // Check if clicked on a cell (works in all views)
    let cellIndex = -1;
    
    if (viewMode === 'lane' && laneCount > 1) {
      // Lane view cell detection
      const COLUMN_OFFSET = 8;
      const HEADER_HEIGHT = 25;
      const scrolledX = mouseX - PADDING - COLUMN_OFFSET + scrollX;
      const relativeY = mouseY - HEADER_HEIGHT - 5 + scrollY;
      
      const col = Math.floor(scrolledX / (CELL_WIDTH + CELL_GAP));
      const lane = Math.floor(relativeY / (CELL_HEIGHT + 4));
      
      if (col >= 0 && lane >= 0 && lane < laneCount) {
        const columnsCount = Math.ceil(tape.length / laneCount);
        if (col < columnsCount) {
          cellIndex = col * laneCount + lane;
        }
      }
    } else {
      // Normal/compact view cell detection
      const x = mouseX + scrollX;
      const cellX = x - PADDING;
      
      if (cellX >= 0) {
        const totalCellWidth = CELL_WIDTH + CELL_GAP;
        cellIndex = Math.floor(cellX / totalCellWidth);
      }
    }
    
    if (cellIndex >= 0 && cellIndex < tape.length) {
      setContextMenu({
        x: e.clientX,
        y: e.clientY,
        type: 'cell',
        index: cellIndex
      });
      return;
    }
    
    setContextMenu(null);
  }, [viewMode, laneCount, scrollX, scrollY, tape.length, CELL_WIDTH, CELL_HEIGHT, CELL_GAP, PADDING]);
  
  // Handle mouse move for scroll bar dragging
  const handleMouseMoveGlobal = useCallback((e: MouseEvent) => {
    // Handle horizontal scroll bar dragging
    if (isDraggingScrollBar && scrollBarDragStart) {
      const deltaX = e.clientX - scrollBarDragStart.mouseX;
      
      // Calculate max scroll based on view mode
      let maxScroll: number;
      let totalVirtualWidth: number;
      
      if (viewMode === 'lane' && laneCount > 1) {
        const columnsCount = Math.ceil(tape.length / laneCount);
        const COLUMN_OFFSET = 8; // Same offset as in drawLaneView
        totalVirtualWidth = PADDING + COLUMN_OFFSET + columnsCount * (CELL_WIDTH + CELL_GAP) + PADDING;
        maxScroll = Math.max(0, totalVirtualWidth - width);
      } else {
        totalVirtualWidth = totalWidth;
        maxScroll = Math.max(0, totalWidth - width);
      }
      
      // Calculate new scroll position
      const scrollRatio = deltaX / width;
      const scrollDelta = scrollRatio * totalVirtualWidth;
      const newScroll = scrollBarDragStart.scrollX + scrollDelta;
      
      setScrollX(Math.max(0, Math.min(newScroll, maxScroll)));
    }
    
    // Handle vertical scroll bar dragging
    if (isDraggingVerticalScrollBar && verticalScrollBarDragStart) {
      const deltaY = e.clientY - verticalScrollBarDragStart.mouseY;
      
      const HEADER_HEIGHT = 25;
      const totalLanesHeight = laneCount * (CELL_HEIGHT + 4);
      const viewportHeight = height - HEADER_HEIGHT - 20;
      const maxScrollY = Math.max(0, totalLanesHeight - viewportHeight);
      
      // Calculate new scroll position
      const scrollRatio = deltaY / viewportHeight;
      const scrollDelta = scrollRatio * totalLanesHeight;
      const newScrollY = verticalScrollBarDragStart.scrollY + scrollDelta;
      
      setScrollY(Math.max(0, Math.min(newScrollY, maxScrollY)));
    }
  }, [isDraggingScrollBar, scrollBarDragStart, isDraggingVerticalScrollBar, verticalScrollBarDragStart, width, height, totalWidth, viewMode, laneCount, tape.length, CELL_WIDTH, CELL_HEIGHT, CELL_GAP, PADDING]);
  
  // Handle mouse up for scroll bar dragging
  const handleMouseUpGlobal = useCallback(() => {
    setIsDraggingScrollBar(false);
    setScrollBarDragStart(null);
    setIsDraggingVerticalScrollBar(false);
    setVerticalScrollBarDragStart(null);
  }, []);
  
  // Add global mouse event listeners for dragging
  useEffect(() => {
    if (isDraggingScrollBar || isDraggingVerticalScrollBar) {
      window.addEventListener('mousemove', handleMouseMoveGlobal);
      window.addEventListener('mouseup', handleMouseUpGlobal);
      
      return () => {
        window.removeEventListener('mousemove', handleMouseMoveGlobal);
        window.removeEventListener('mouseup', handleMouseUpGlobal);
      };
    }
  }, [isDraggingScrollBar, isDraggingVerticalScrollBar, handleMouseMoveGlobal, handleMouseUpGlobal]);
  
  // Handle scroll-to events from parent component
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const handleScrollToIndex = (e: Event) => {
      const customEvent = e as CustomEvent;
      const targetIndex = customEvent.detail.index;
      
      let targetX: number;
      let effectiveWidth: number;
      
      if (viewMode === 'lane' && laneCount > 1) {
        // In lane view, calculate based on columns
        const targetColumn = Math.floor(targetIndex / laneCount);
        const COLUMN_OFFSET = 8; // Same offset as in drawLaneView
        targetX = PADDING + COLUMN_OFFSET + targetColumn * (CELL_WIDTH + CELL_GAP);
        const columnsCount = Math.ceil(tape.length / laneCount);
        effectiveWidth = PADDING + COLUMN_OFFSET + columnsCount * (CELL_WIDTH + CELL_GAP) + PADDING;
      } else {
        targetX = PADDING + targetIndex * (CELL_WIDTH + CELL_GAP);
        effectiveWidth = totalWidth;
      }
      
      // Center the target in view
      const targetScroll = targetX - width / 2 + CELL_WIDTH / 2;
      const newScroll = Math.max(0, Math.min(targetScroll, effectiveWidth - width));
      setScrollX(newScroll);
    };
    
    canvas.addEventListener('scrollToIndex', handleScrollToIndex);
    
    return () => {
      canvas.removeEventListener('scrollToIndex', handleScrollToIndex);
    };
  }, [viewMode, laneCount, tape.length, width, totalWidth, CELL_WIDTH, CELL_GAP, PADDING]);
  
  // Context menu component
  const renderContextMenu = () => {
    if (!contextMenu) return null;
    
    const currentLabel = contextMenu.type === 'lane' 
      ? labels.lanes[contextMenu.index] 
      : contextMenu.type === 'column'
      ? labels.columns[contextMenu.index]
      : labels.cells[contextMenu.index];
    
    return (
      <>
        {/* Backdrop to close menu when clicking outside */}
        <div 
          className="fixed inset-0 z-40" 
          onClick={() => setContextMenu(null)}
        />
        
        <div
          className="fixed z-50 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl overflow-hidden"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          {/* Header */}
          <div className="px-3 py-2 bg-zinc-800 border-b border-zinc-700">
            <h3 className="text-xs font-medium text-zinc-300">
              Edit {contextMenu.type === 'lane' ? 'Lane' : contextMenu.type === 'column' ? 'Column' : 'Cell'} Label
            </h3>
            <p className="text-[10px] text-zinc-500 mt-0.5">
              {contextMenu.type === 'lane' ? 'Lane' : contextMenu.type === 'column' ? 'Column' : 'Cell'} #{contextMenu.index}
              {currentLabel && <span className="text-zinc-400"> • Current: "{currentLabel}"</span>}
            </p>
          </div>
          
          {/* Input section */}
          <div className="p-3">
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder={`Enter ${contextMenu.type} label...`}
                defaultValue={currentLabel || ''}
                className="flex-1 px-2 py-1.5 text-sm bg-zinc-800 border border-zinc-700 rounded focus:border-zinc-600 focus:outline-none"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    const value = (e.target as HTMLInputElement).value.trim();
                    if (value) {
                      if (contextMenu.type === 'lane') {
                        tapeLabelsStore.setLaneLabel(contextMenu.index, value);
                      } else if (contextMenu.type === 'column') {
                        tapeLabelsStore.setColumnLabel(contextMenu.index, value);
                      } else {
                        tapeLabelsStore.setCellLabel(contextMenu.index, value);
                      }
                    } else if (currentLabel) {
                      // Only remove if there was a label before
                      if (contextMenu.type === 'lane') {
                        tapeLabelsStore.removeLaneLabel(contextMenu.index);
                      } else if (contextMenu.type === 'column') {
                        tapeLabelsStore.removeColumnLabel(contextMenu.index);
                      } else {
                        tapeLabelsStore.removeCellLabel(contextMenu.index);
                      }
                    }
                    setContextMenu(null);
                  } else if (e.key === 'Escape') {
                    setContextMenu(null);
                  }
                }}
              />
              {currentLabel && (
                <button
                  className="p-1.5 rounded hover:bg-zinc-800 transition-colors group"
                  onClick={() => {
                    if (contextMenu.type === 'lane') {
                      tapeLabelsStore.removeLaneLabel(contextMenu.index);
                    } else if (contextMenu.type === 'column') {
                      tapeLabelsStore.removeColumnLabel(contextMenu.index);
                    } else {
                      tapeLabelsStore.removeCellLabel(contextMenu.index);
                    }
                    setContextMenu(null);
                  }}
                  title="Remove label"
                >
                  <XMarkIcon className="w-4 h-4 text-zinc-500 group-hover:text-red-400" />
                </button>
              )}
            </div>
            <p className="text-[10px] text-zinc-500 mt-2">
              Press Enter to save, Escape to cancel
            </p>
          </div>
        </div>
      </>
    );
  };

  return (
    <>
      <canvas
        ref={canvasRef}
        className="cursor-pointer"
        onWheel={handleWheel}
        onMouseMove={handleMouseMove}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onMouseDown={handleMouseDown}
        onContextMenu={handleContextMenu}
        style={{ cursor: isDraggingScrollBar || isDraggingVerticalScrollBar ? 'grabbing' : 'pointer' }}
      />
      {renderContextMenu()}
    </>
  );
}