import { useState, useEffect } from 'react';
import { XMarkIcon } from '@heroicons/react/24/solid';
import { SparklesIcon } from '@heroicons/react/24/outline';

interface LearningNoticeProps {
  activeTab: string | null;
}

export function LearningNotice({ activeTab }: LearningNoticeProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isDismissed, setIsDismissed] = useState(false);

  useEffect(() => {
    // Check if this is the first visit
    const hasVisited = localStorage.getItem('hasVisitedBefore');
    const learningNoticeDismissed = localStorage.getItem(
      'learningNoticeDismissed',
    );

    if (!hasVisited) {
      // Mark as visited for future sessions
      localStorage.setItem('hasVisitedBefore', 'true');

      // Show notice if not previously dismissed
      if (learningNoticeDismissed !== 'true') {
        // Add small delay for better UX
        setTimeout(() => setIsVisible(true), 500);
      }
    }
  }, []);

  useEffect(() => {
    // Hide and dismiss when Learning tab is opened
    if (activeTab === 'learning' && isVisible) {
      handleDismiss();
    }
  }, [activeTab, isVisible]);

  const handleDismiss = () => {
    setIsDismissed(true);
    setIsVisible(false);
    localStorage.setItem('learningNoticeDismissed', 'true');
  };

  if (!isVisible || isDismissed) {
    return null;
  }

  return (
    <div
      className="absolute bottom-28 left-14 z-50"
      style={{
        animation: 'slide-in-left 0.3s ease-out',
      }}
    >
      <div className="relative bg-gradient-to-r from-purple-900/90 to-indigo-900/90 backdrop-blur-sm rounded-lg shadow-xl border border-purple-700/50 max-w-xs">
        {/* Arrow pointing to Learning button */}
        <div
          className="absolute -left-2 bottom-4 w-0 h-0 
                    border-t-[8px] border-t-transparent
                    border-r-[12px] border-r-purple-900/90
                    border-b-[8px] border-b-transparent"
        ></div>

        <div className="p-4">
          <div className="flex items-start gap-3">
            <SparklesIcon className="w-5 h-5 text-purple-300 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <p className="text-sm text-purple-100 font-semibold mb-1">
                New to Braintease IDE?
              </p>
              <p className="text-xs text-purple-200">
                Check out the Learning panel for tutorials and examples
              </p>
            </div>
            <button
              onClick={handleDismiss}
              className="p-1 hover:bg-purple-800/50 rounded transition-colors flex-shrink-0"
              aria-label="Dismiss notice"
            >
              <XMarkIcon className="w-4 h-4 text-purple-300" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
