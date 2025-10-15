import { useState, useEffect } from 'react';
import { XMarkIcon } from '@heroicons/react/24/solid';
import { ExclamationTriangleIcon } from '@heroicons/react/24/outline';

export function BrowserNotice() {
  const [isChromeBased, setIsChromeBased] = useState(true);
  const [isDismissed, setIsDismissed] = useState(false);

  useEffect(() => {
    // Check if browser is Chrome-based
    const userAgent = navigator.userAgent.toLowerCase();
    const isChrome = userAgent.includes('chrome');
    const isChromium = userAgent.includes('chromium');
    const isEdge = userAgent.includes('edg/'); // Edge is Chromium-based
    const isOpera = userAgent.includes('opr/'); // Opera is Chromium-based
    const isBrave = (navigator as any).brave?.isBrave?.name === 'isBrave';

    // Safari check (not Chrome-based)
    const isSafari = userAgent.includes('safari') && !isChrome;
    // Firefox check
    const isFirefox = userAgent.includes('firefox');

    const isChromiumBased =
      isChrome || isChromium || isEdge || isOpera || isBrave;
    setIsChromeBased(isChromiumBased);

    // Check if already dismissed in this session
    const dismissed = sessionStorage.getItem('browserNoticeDismissed');
    if (dismissed === 'true') {
      setIsDismissed(true);
    }
  }, []);

  const handleDismiss = () => {
    setIsDismissed(true);
    sessionStorage.setItem('browserNoticeDismissed', 'true');
  };

  // Don't show if Chrome-based or dismissed
  if (isChromeBased || isDismissed) {
    return null;
  }

  return (
    <div className="fixed top-0 left-0 right-0 bg-amber-900/20 border-b border-amber-700/30 z-50 backdrop-blur-sm">
      <div className="px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <ExclamationTriangleIcon className="w-5 h-5 text-amber-500 flex-shrink-0" />
          <p className="text-sm text-amber-200">
            <span className="font-semibold">Browser Compatibility Notice:</span>{' '}
            This IDE was not tested in non-Chrome-based browsers. Some features,
            particularly the code editor, may not work correctly in your current
            browser. For the best experience, please use Chrome, Edge, Brave, or
            another Chromium-based browser.
          </p>
        </div>
        <button
          onClick={handleDismiss}
          className="p-1 hover:bg-amber-800/30 rounded transition-colors ml-4 flex-shrink-0"
          aria-label="Dismiss notice"
        >
          <XMarkIcon className="w-5 h-5 text-amber-400" />
        </button>
      </div>
    </div>
  );
}
