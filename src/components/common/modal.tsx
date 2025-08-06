import { useEffect, useRef, ReactNode } from "react";
import clsx from "clsx";

interface ModalProps {
    isOpen: boolean;
    onClose: () => void;
    children: ReactNode;
    className?: string;
    showBackdrop?: boolean;
    backdropClassName?: string;
    width?: string;
    maxHeight?: string;
    position?: "center" | "top";
}

export function Modal({
    isOpen,
    onClose,
    children,
    className,
    showBackdrop = true,
    backdropClassName = "bg-black/30",
    width = "w-[600px]",
    maxHeight = "max-h-[400px]",
    position = "center"
}: ModalProps) {
    const modalRef = useRef<HTMLDivElement>(null);

    // Handle escape key
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            }
        };

        document.addEventListener('keydown', handleKeyDown, true);
        return () => document.removeEventListener('keydown', handleKeyDown, true);
    }, [isOpen, onClose]);

    // Handle click outside
    useEffect(() => {
        if (!isOpen || !showBackdrop) return;

        const handleClickOutside = (e: MouseEvent) => {
            // Check if click is on the backdrop (not on the modal content)
            const target = e.target as HTMLElement;
            if (target.classList.contains('modal-backdrop')) {
                e.preventDefault();
                e.stopPropagation();
                onClose();
            }
        };

        // Small delay to avoid closing on the same click that opened it
        setTimeout(() => {
            document.addEventListener('mousedown', handleClickOutside);
        }, 0);

        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [isOpen, onClose, showBackdrop]);

    // Focus modal when opened
    useEffect(() => {
        if (isOpen && modalRef.current) {
            modalRef.current.focus();
        }
    }, [isOpen]);

    if (!isOpen) return null;

    const positionClasses = position === "top" ? "items-start pt-24" : "items-center";

    return (
        <div 
            className={clsx("fixed inset-0 z-50 flex justify-center", positionClasses)}
            onKeyDown={(e) => {
                // Prevent browser shortcuts while modal is open
                if (e.metaKey || e.ctrlKey) {
                    e.preventDefault();
                }
            }}
        >
            {/* Backdrop */}
            {showBackdrop && (
                <div 
                    className={clsx("absolute inset-0 modal-backdrop", backdropClassName)}
                />
            )}
            
            {/* Modal */}
            <div 
                ref={modalRef}
                className={clsx(
                    "relative bg-zinc-900 border border-zinc-700 rounded-lg shadow-2xl flex flex-col outline-none",
                    width,
                    maxHeight,
                    className
                )}
                tabIndex={-1}
            >
                {children}
            </div>
        </div>
    );
}



// Common modal header component
interface ModalHeaderProps {
    title: string;
    onClose?: () => void;
    className?: string;
}

export function ModalHeader({ title, onClose, className }: ModalHeaderProps) {
    return (
        <div className={clsx("p-3 border-b border-zinc-700 flex items-center justify-between", className)}>
            <h2 className="text-lg font-semibold text-zinc-100">{title}</h2>
            {onClose && (
                <button
                    onClick={onClose}
                    className="text-zinc-400 hover:text-zinc-100 transition-colors p-1 rounded hover:bg-zinc-800"
                    aria-label="Close"
                >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            )}
        </div>
    );
}

// Common modal footer component
interface ModalFooterProps {
    children: ReactNode;
    className?: string;
}

export function ModalFooter({ children, className }: ModalFooterProps) {
    return (
        <div className={clsx("p-2 border-t border-zinc-700 text-xs text-zinc-500", className)}>
            {children}
        </div>
    );
}