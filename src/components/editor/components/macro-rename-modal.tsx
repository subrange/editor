import { useEffect, useRef, useState } from 'react';
import { Modal, ModalFooter } from '../../common/modal.tsx';
import clsx from 'clsx';

interface MacroRenameModalProps {
  isOpen: boolean;
  currentName: string;
  onClose: () => void;
  onRename: (newName: string) => void;
  existingMacroNames: string[];
}

export function MacroRenameModal({
  isOpen,
  currentName,
  onClose,
  onRename,
  existingMacroNames,
}: MacroRenameModalProps) {
  const [newName, setNewName] = useState(currentName);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Reset state when modal opens
  useEffect(() => {
    if (isOpen) {
      setNewName(currentName);
      setError(null);
      // Focus and select all text when modal opens
      setTimeout(() => {
        if (inputRef.current) {
          inputRef.current.focus();
          inputRef.current.select();
        }
      }, 0);
    }
  }, [isOpen, currentName]);

  // Validate the new name
  const validateName = (name: string): string | null => {
    if (!name.trim()) {
      return 'Macro name cannot be empty';
    }

    if (!/^[a-zA-Z_]\w*$/.test(name)) {
      return 'Macro name must start with a letter or underscore and contain only letters, numbers, and underscores';
    }

    if (name !== currentName && existingMacroNames.includes(name)) {
      return `Macro "${name}" already exists`;
    }

    return null;
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setNewName(value);
    setError(validateName(value));
  };

  const handleSubmit = () => {
    const validationError = validateName(newName);
    if (validationError) {
      setError(validationError);
      return;
    }

    if (newName !== currentName) {
      onRename(newName);
    }
    onClose();
  };

  // Handle keyboard events
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Enter' && !error) {
        e.preventDefault();
        handleSubmit();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, newName, error]);

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      position="center"
      width="w-[400px]"
      maxHeight="max-h-[200px]"
    >
      {/* Header */}
      <div className="p-3 border-b border-zinc-700">
        <div className="text-sm text-zinc-400">
          Rename macro <span className="text-blue-400">@{currentName}</span>
        </div>
      </div>

      {/* Content */}
      <div className="p-4">
        <input
          ref={inputRef}
          type="text"
          value={newName}
          onChange={handleInputChange}
          className={clsx(
            'w-full bg-zinc-800 text-zinc-100 px-3 py-2 rounded text-sm outline-none font-mono',
            'focus:ring-1',
            error ? 'focus:ring-red-500' : 'focus:ring-blue-500',
          )}
          placeholder="Enter new macro name"
        />
        {error && <div className="mt-2 text-xs text-red-400">{error}</div>}
      </div>

      <ModalFooter>
        <div className="flex items-center justify-between w-full">
          <div className="flex items-center gap-3 text-xs">
            <span>Enter Rename</span>
            <span>Esc Cancel</span>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-3 py-1 text-xs text-zinc-400 hover:text-zinc-100 transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSubmit}
              disabled={!!error || newName === currentName}
              className={clsx(
                'px-3 py-1 text-xs rounded transition-colors',
                error || newName === currentName
                  ? 'bg-zinc-800 text-zinc-600 cursor-not-allowed'
                  : 'bg-blue-600 text-white hover:bg-blue-700',
              )}
            >
              Rename
            </button>
          </div>
        </div>
      </ModalFooter>
    </Modal>
  );
}
