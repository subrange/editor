import { useRef, useState, useEffect, useCallback } from 'react';
import clsx from 'clsx';

interface DraggableVSepProps {
  onResize: (leftWidth: number) => void;
  minLeftWidth?: number;
  minRightWidth?: number;
  className?: string;
}

export function DraggableVSep({
  onResize,
  minLeftWidth = 200,
  minRightWidth = 200,
  className = '',
}: DraggableVSepProps) {
  const [isDragging, setIsDragging] = useState(false);
  const separatorRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!separatorRef.current) return;

      const parent = separatorRef.current.parentElement;
      if (!parent) return;

      const parentRect = parent.getBoundingClientRect();
      const newLeftWidth = e.clientX - parentRect.left;

      // Enforce minimum widths
      const maxLeftWidth = parentRect.width - minRightWidth;
      const clampedWidth = Math.max(
        minLeftWidth,
        Math.min(newLeftWidth, maxLeftWidth),
      );

      onResize(clampedWidth);
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, onResize, minLeftWidth, minRightWidth]);

  return (
    <div
      ref={separatorRef}
      className={clsx(
        'vsep bg-zinc-800 relative hover:bg-zinc-700 transition-colors',
        {
          'bg-zinc-700': isDragging,
          [className]: className.length > 0,
        },
      )}
    >
      {/* Invisible wider hit area for easier grabbing */}
      <div
        className="absolute inset-y-0 -left-[8px] -right-[8px] cursor-col-resize"
        onMouseDown={handleMouseDown}
        style={{ zIndex: 10 }}
      />
    </div>
  );
}
