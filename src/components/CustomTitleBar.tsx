import { useState } from 'react';
import { X } from 'lucide-react';

const CustomTitleBar = () => {
  const [isMaximized, setIsMaximized] = useState(false);

  const minimize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      await window.minimize();
    } catch (err) {
      console.error('[ERROR] Minimize failed:', err);
      alert('Minimize failed: ' + err);
    }
  };

  const toggleMaximize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const appWindow = getCurrentWindow();
      const currentMaximized = await appWindow.isMaximized();
      await appWindow.toggleMaximize();
      const newMaximized = await appWindow.isMaximized();
      setIsMaximized(newMaximized ?? currentMaximized);
    } catch (err) {
      console.error('[ERROR] Maximize failed:', err);
      alert('Maximize failed: ' + err);
    }
  };

  const close = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      await window.close();
    } catch (err) {
      console.error('[ERROR] Close failed:', err);
      alert('Close failed: ' + err);
    }
  };

  return (
    <div
      className="fixed top-0 left-0 right-0 h-8 bg-black/98 backdrop-blur-xl border-b border-gray-900/50 flex items-center justify-between px-3 z-[99999] select-none"
      data-tauri-drag-region
    >
      <div className="flex items-center gap-2 flex-1 h-full">
        <div className="w-5 h-5 relative pointer-events-none">
          <img src="/128x128.png" alt="" className="w-full h-full object-contain" />
        </div>
        <span className="text-xs font-semibold text-gray-500">ColorWall</span>
      </div>

      <div className="flex items-center relative z-10">
        <button
          onClick={minimize}
          className="w-12 h-8 flex items-center justify-center hover:bg-gray-800/50 transition-colors text-gray-400 hover:text-gray-200 cursor-default"
          style={{ WebkitAppRegion: 'no-drag' } as any}
        >
          <span className="text-xl leading-none mb-1">−</span>
        </button>
        <button
          onClick={toggleMaximize}
          className="w-12 h-8 flex items-center justify-center hover:bg-gray-800/50 transition-colors text-gray-400 hover:text-gray-200 cursor-default"
          style={{ WebkitAppRegion: 'no-drag' } as any}
        >
          <span className="text-sm leading-none">{isMaximized ? '❐' : '□'}</span>
        </button>
        <button
          onClick={close}
          className="w-12 h-8 flex items-center justify-center hover:bg-red-600 transition-colors text-gray-400 hover:text-white cursor-default"
          style={{ WebkitAppRegion: 'no-drag' } as any}
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};

export default CustomTitleBar;

