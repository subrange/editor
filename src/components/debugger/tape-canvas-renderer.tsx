import { useEffect, useRef, useState, useCallback } from 'react';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe';
import { interpreterStore } from './interpreter-facade.store';
import { settingsStore } from '../../stores/settings.store';

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
  
  const tape = interpreterState.tape;
  const pointer = interpreterState.pointer;
  const cellBits = tape instanceof Uint8Array ? 8 : tape instanceof Uint16Array ? 16 : 32;
  
  // Scroll state
  const [scrollX, setScrollX] = useState(0);
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const [hoveredColumn, setHoveredColumn] = useState<number | null>(null);
  const [hoveredLane, setHoveredLane] = useState<number | null>(null);
  
  // Animation state for smooth transitions
  const animationStateRef = useRef<Map<number, { 
    currentOpacity: number; 
    targetOpacity: number;
    lastUpdate: number;
  }>>(new Map());
  
  // Cell dimensions based on view mode
  const dimensions = viewMode === 'compact' ? {
    cellWidth: 60,
    cellHeight: 40,
    cellGap: 2,
    padding: 12,
    fontSize: { index: 9, value: 16, binary: 8, ascii: 10 }
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
    
    // Save context state
    ctx.save();
    
    // Apply scroll transform
    ctx.translate(-scrollX, 0);
    
    // Draw visible cells
    for (let i = firstVisibleIndex; i <= lastVisibleIndex; i++) {
      const x = PADDING + i * (CELL_WIDTH + CELL_GAP);
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
      
      // Draw cell index
      if (dimensions.fontSize.index > 0) {
        ctx.fillStyle = isPointer ? 
          `rgba(250, 204, 21, ${opacity})` : 
          `rgba(113, 113, 122, ${opacity})`; // yellow-400 : zinc-500
        ctx.font = `${dimensions.fontSize.index}px monospace`;
        ctx.textAlign = 'center';
        ctx.fillText(`#${i}`, x + CELL_WIDTH / 2, y + (viewMode === 'compact' ? 15 : 20));
      }
      
      // Draw value
      ctx.fillStyle = isPointer ? 
        `rgba(253, 224, 71, ${opacity})` : 
        value !== 0 ? `rgba(147, 197, 253, ${opacity})` : 
        `rgba(161, 161, 170, ${opacity})`; // yellow-300 : blue-300 : zinc-400
      ctx.font = `bold ${dimensions.fontSize.value}px monospace`;
      ctx.textAlign = 'center';
      ctx.fillText(value.toString(), x + CELL_WIDTH / 2, y + CELL_HEIGHT / 2 + (viewMode === 'compact' ? 4 : 8));
      
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
    
    // Restore context state
    ctx.restore();
    
    // Draw scroll indicator at bottom
    if (totalWidth > width) {
      const scrollBarHeight = 4;
      const scrollBarY = height - scrollBarHeight - 10;
      const scrollBarWidth = Math.max(50, (width / totalWidth) * width);
      const scrollBarX = (scrollX / (totalWidth - width)) * (width - scrollBarWidth);
      
      // Scroll track
      ctx.fillStyle = '#27272a'; // zinc-800
      ctx.fillRect(0, scrollBarY, width, scrollBarHeight);
      
      // Scroll thumb
      ctx.fillStyle = '#52525b'; // zinc-600
      ctx.fillRect(scrollBarX, scrollBarY, scrollBarWidth, scrollBarHeight);
    }
    
    // Draw stats in corner
    ctx.fillStyle = '#a1a1aa'; // zinc-400
    ctx.font = '11px monospace';
    ctx.textAlign = 'left';
    ctx.fillText(`Memory: ${tape.length.toLocaleString()} cells | Pointer: ${pointer} | Value: ${tape[pointer]}`, 10, 20);
  }, [width, height, scrollX, tape, pointer, cellBits, firstVisibleIndex, lastVisibleIndex, hoveredIndex, totalWidth, viewMode, dimensions, getAnimatedOpacity]);
  
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
    
    // Visible columns
    const firstColumn = Math.max(0, Math.floor((scrollX - PADDING) / columnWidth));
    const lastColumn = Math.min(columnsCount - 1, Math.ceil((scrollX + width - PADDING) / columnWidth));
    
    // Set up clipping region for scrollable content
    ctx.save();
    ctx.beginPath();
    ctx.rect(PADDING, 0, width - PADDING, height);
    ctx.clip();
    
    // Draw scrollable content
    ctx.translate(-scrollX, 0);
    
    // Draw column headers (scrollable)
    ctx.fillStyle = '#18181b';
    ctx.fillRect(PADDING, 0, columnsCount * columnWidth, 25);
    for (let col = firstColumn; col <= lastColumn; col++) {
      const x = PADDING + col * columnWidth + CELL_WIDTH / 2;
      ctx.fillStyle = '#71717a';
      ctx.font = '10px monospace';
      ctx.textAlign = 'center';
      ctx.fillText(col.toString(), x, 18);
    }
    
    // Draw cells
    for (let col = firstColumn; col <= lastColumn; col++) {
      for (let lane = 0; lane < laneCount; lane++) {
        const index = col * laneCount + lane;
        if (index >= tape.length) continue;
        
        const x = PADDING + col * columnWidth;
        const y = 30 + lane * laneHeight;
        const value = tape[index];
        const isPointer = index === pointer;
        const isHovered = index === hoveredIndex;
        const laneColor = LANE_COLORS[lane % LANE_COLORS.length];
        const isDimmed = hoveredIndex !== null && hoveredColumn !== null && hoveredLane !== null && 
                        (col !== hoveredColumn && lane !== hoveredLane);
        
        // Get animated opacity
        const opacity = getAnimatedOpacity(index, isDimmed);
        
        // Cell background with animated opacity
        if (isPointer) {
          ctx.fillStyle = `rgba(234, 179, 8, ${0.2 * opacity})`;
          ctx.strokeStyle = `rgba(234, 179, 8, ${0.5 * opacity})`;
          ctx.lineWidth = 2;
        } else {
          // Parse the lane color and apply opacity
          const rgbaMatch = laneColor.fill.match(/rgba?\((\d+),\s*(\d+),\s*(\d+),?\s*([\d.]+)?\)/);
          if (rgbaMatch) {
            const [_, r, g, b] = rgbaMatch;
            ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${0.1 * opacity})`;
          } else {
            ctx.fillStyle = laneColor.fill;
          }
          
          const strokeMatch = laneColor.stroke.match(/#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})/i);
          if (strokeMatch) {
            const [_, r, g, b] = strokeMatch;
            const rDec = parseInt(r, 16);
            const gDec = parseInt(g, 16);
            const bDec = parseInt(b, 16);
            ctx.strokeStyle = `rgba(${rDec}, ${gDec}, ${bDec}, ${0.4 * opacity})`;
          } else {
            ctx.strokeStyle = laneColor.stroke;
          }
          ctx.lineWidth = 1;
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
        
        // Cell content with animated opacity
        ctx.fillStyle = `rgba(113, 113, 122, ${opacity})`;
        ctx.font = '9px monospace';
        ctx.textAlign = 'left';
        ctx.fillText(index.toString(), x + 3, y + CELL_HEIGHT / 2 - 2);
        
        ctx.fillStyle = isPointer ? 
          `rgba(253, 224, 71, ${opacity})` : 
          value !== 0 ? `rgba(147, 197, 253, ${opacity})` : 
          `rgba(161, 161, 170, ${opacity})`;
        ctx.font = 'bold 12px monospace';
        ctx.textAlign = 'right';
        ctx.fillText(value.toString(), x + CELL_WIDTH - 3, y + CELL_HEIGHT / 2 + 4);
      }
    }
    
    ctx.restore();
    
    // Draw fixed lane headers (not scrolled)
    ctx.fillStyle = '#18181b'; // zinc-900
    ctx.fillRect(0, 0, PADDING, height);
    
    // Draw corner cell
    ctx.fillStyle = '#09090b'; // zinc-950
    ctx.fillRect(0, 0, PADDING, 25);
    ctx.strokeStyle = '#3f3f46'; // zinc-700
    ctx.lineWidth = 1;
    ctx.strokeRect(0, 0, PADDING, 25);
    ctx.fillStyle = '#71717a';
    ctx.font = '9px monospace';
    ctx.textAlign = 'center';
    ctx.fillText('L/C', PADDING / 2, 16);
    
    // Draw lane numbers
    for (let lane = 0; lane < laneCount; lane++) {
      const y = 30 + lane * laneHeight;
      
      // Subtle background for lane rows
      if (lane % 2 === 0) {
        ctx.fillStyle = 'rgba(63, 63, 70, 0.1)'; // zinc-700/10
        ctx.fillRect(0, y, PADDING, laneHeight - 4);
      }
      
      // Lane number
      ctx.fillStyle = '#a1a1aa';
      ctx.font = '11px monospace';
      ctx.textAlign = 'center';
      ctx.fillText(lane.toString(), PADDING / 2, y + laneHeight / 2 + 3);
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
    ctx.moveTo(0, 25);
    ctx.lineTo(width, 25);
    ctx.stroke();
    
    // Draw scroll indicator
    if (columnsCount * columnWidth > width) {
      const scrollBarHeight = 4;
      const scrollBarY = height - scrollBarHeight - 10;
      const totalVirtualWidth = PADDING + columnsCount * columnWidth + PADDING;
      const scrollBarWidth = Math.max(50, (width / totalVirtualWidth) * width);
      const scrollBarX = (scrollX / (totalVirtualWidth - width)) * (width - scrollBarWidth);
      
      ctx.fillStyle = '#27272a';
      ctx.fillRect(0, scrollBarY, width, scrollBarHeight);
      
      ctx.fillStyle = '#52525b';
      ctx.fillRect(scrollBarX, scrollBarY, scrollBarWidth, scrollBarHeight);
    }
  }, [tape, pointer, laneCount, scrollX, width, height, PADDING, CELL_WIDTH, CELL_HEIGHT, CELL_GAP, hoveredIndex, hoveredColumn, hoveredLane, getAnimatedOpacity]);
  
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
  
  // Auto-scroll to pointer
  useEffect(() => {
    let pointerX: number;
    let effectiveWidth: number;
    
    if (viewMode === 'lane' && laneCount > 1) {
      // In lane view, calculate based on columns
      const pointerColumn = Math.floor(pointer / laneCount);
      pointerX = PADDING + pointerColumn * (CELL_WIDTH + CELL_GAP);
      const columnsCount = Math.ceil(tape.length / laneCount);
      effectiveWidth = PADDING + columnsCount * (CELL_WIDTH + CELL_GAP) + PADDING;
    } else {
      pointerX = PADDING + pointer * (CELL_WIDTH + CELL_GAP);
      effectiveWidth = totalWidth;
    }
    
    const pointerInView = pointerX >= scrollX && pointerX + CELL_WIDTH <= scrollX + width;
    
    if (!pointerInView) {
      // Center the pointer in view
      const targetScroll = pointerX - width / 2 + CELL_WIDTH / 2;
      setScrollX(Math.max(0, Math.min(targetScroll, effectiveWidth - width)));
    }
  }, [pointer, width, totalWidth, viewMode, laneCount, tape.length, CELL_WIDTH, CELL_GAP, PADDING]);
  
  // Handle mouse wheel scrolling
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaX || e.deltaY;
    const newScroll = scrollX + delta;
    
    let maxScroll: number;
    if (viewMode === 'lane' && laneCount > 1) {
      const columnsCount = Math.ceil(tape.length / laneCount);
      const laneViewWidth = PADDING + columnsCount * (CELL_WIDTH + CELL_GAP) + PADDING;
      maxScroll = Math.max(0, laneViewWidth - width);
    } else {
      maxScroll = Math.max(0, totalWidth - width);
    }
    
    setScrollX(Math.max(0, Math.min(newScroll, maxScroll)));
  }, [scrollX, totalWidth, width, viewMode, laneCount, tape.length, CELL_WIDTH, CELL_GAP, PADDING]);
  
  // Handle mouse move for hover
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;
    
    if (viewMode === 'lane' && laneCount > 1) {
      // Lane view hover detection
      if (mouseX < PADDING || mouseY < 25) {
        setHoveredIndex(null);
        setHoveredColumn(null);
        setHoveredLane(null);
        return;
      }
      
      const scrolledX = mouseX - PADDING + scrollX;
      const relativeY = mouseY - 30;
      
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
  }, [scrollX, tape.length, viewMode, laneCount, CELL_WIDTH, CELL_HEIGHT, CELL_GAP, PADDING]);
  
  const handleMouseLeave = useCallback(() => {
    setHoveredIndex(null);
    setHoveredColumn(null);
    setHoveredLane(null);
  }, []);
  
  return (
    <canvas
      ref={canvasRef}
      className="cursor-pointer"
      onWheel={handleWheel}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
    />
  );
}