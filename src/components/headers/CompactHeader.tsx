// import React from 'react';
import { HardDrive } from 'lucide-react';
import LaxentaLogo from '../LaxentaLogo';

interface CompactHeaderProps {
  cacheInfo: { sizeMB: string; fileCount: number };
  isHeaderCompact: boolean;
  onExpand: () => void;
}

const CompactHeader = ({ cacheInfo, isHeaderCompact, onExpand }: CompactHeaderProps) => {
  return (
    <div
      className={`fixed top-8 left-0 right-0 z-50 transition-all duration-500 ease-out ${
        isHeaderCompact ? 'translate-y-0 opacity-100' : '-translate-y-full opacity-0 pointer-events-none'
      }`}
      data-tauri-drag-region
    >
      <div className="bg-black/98 backdrop-blur-xl border-b border-gray-900 shadow-2xl">
        <div className="max-w-7xl mx-auto px-4 py-2.5 flex items-center justify-between">
          <button
            onClick={onExpand}
            className="flex items-center gap-2.5 hover:opacity-80 transition-opacity cursor-pointer"
          >
            <LaxentaLogo />
            <span className="text-base font-bold bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
              ColorWall
            </span>
          </button>

          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-2 bg-gray-950/80 px-3 py-1.5 rounded-lg border border-gray-900">
              <HardDrive className="w-3.5 h-3.5 text-blue-400" />
              <span className="text-xs font-semibold text-gray-400">{cacheInfo.sizeMB} MB</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CompactHeader;

