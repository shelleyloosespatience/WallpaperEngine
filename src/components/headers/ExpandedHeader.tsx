import React from 'react';
import { CheckCircle, HardDrive, Loader2, Search, Trash2, Play, Pause } from 'lucide-react';
import LaxentaLogo from '../LaxentaLogo';
import { SOURCE_OPTIONS } from '../../constants/wallpapers';
import { WallpaperSourceOption } from '../../types/wallpaper';
import { getSourceIcon } from '../icons';

interface ExpandedHeaderProps {
  cacheInfo: { sizeMB: string; fileCount: number };
  selectedSource: WallpaperSourceOption;
  onSourceChange: (value: WallpaperSourceOption) => void;
  searchTags: string;
  onSearchTagsChange: (value: string) => void;
  excludeTags: string;
  onExcludeTagsChange: (value: string) => void;
  maxInputLength: number;
  loading: boolean;
  onSearch: () => void;
  clearCache: () => Promise<void>;
  currentWallpaper: string;
  showExpandedHeader: boolean;
  cacheLabel?: string;
  videoWallpaperState?: { isActive: boolean; videoPath: string | null; videoUrl: string | null };
  isTogglingLive?: boolean;
  onToggleLive?: () => void;
}

const ExpandedHeader = ({
  cacheInfo,
  selectedSource,
  onSourceChange,
  searchTags,
  onSearchTagsChange,
  excludeTags,
  onExcludeTagsChange,
  maxInputLength,
  loading,
  onSearch,
  clearCache,
  currentWallpaper,
  showExpandedHeader,
  cacheLabel = 'Laxenta Inc',
  videoWallpaperState,
  isTogglingLive = false,
  onToggleLive,
}: ExpandedHeaderProps) => {
  return (
    <div
      className={`bg-black/98 backdrop-blur-xl border-b border-gray-900 sticky top-8 z-40 shadow-2xl transition-all duration-500 ease-out ${
        showExpandedHeader ? 'translate-y-0 opacity-100' : '-translate-y-full opacity-0 pointer-events-none'
      }`}
      data-tauri-drag-region
    >
      <div className="max-w-7xl mx-auto px-4 py-4 relative z-10">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2.5">
            <LaxentaLogo />
            <div>
              <h1 className="text-xl font-bold bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
                ColorWall
              </h1>
              <p className="text-xs text-gray-600">{cacheLabel}</p>
              <p className="text-xs text-gray-700">https://laxenta.tech</p>
            </div>
          </div>

          <div className="flex items-center gap-2.5">
            {videoWallpaperState && onToggleLive && (
              <button
                onClick={onToggleLive}
                disabled={isTogglingLive || !videoWallpaperState.videoPath}
                className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-all border text-sm font-medium cursor-pointer ${
                  videoWallpaperState.isActive
                    ? 'bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-400 border-emerald-500/20 hover:border-emerald-500/40'
                    : 'bg-gray-950/80 hover:bg-gray-900 text-gray-400 border-gray-900'
                } ${isTogglingLive || !videoWallpaperState.videoPath ? 'opacity-50 cursor-not-allowed' : ''}`}
              >
                {isTogglingLive ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span>Removing live wallpaper...</span>
                  </>
                ) : videoWallpaperState.isActive ? (
                  <>
                    <Pause className="w-4 h-4" />
                    <span>Live: ON (click to remove)</span>
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4" />
                    <span>Live wallpaper OFF</span>
                  </>
                )}
              </button>
            )}
            <div className="flex items-center gap-2 bg-gray-950/80 backdrop-blur-sm px-3 py-2 rounded-lg border border-gray-900">
              <HardDrive className="w-4 h-4 text-blue-400" />
              <span className="text-sm font-semibold text-gray-400">{cacheInfo.sizeMB} MB</span>
              <span className="text-xs text-gray-600">({cacheInfo.fileCount})</span>
            </div>
            <button
              onClick={clearCache}
              className="flex items-center gap-2 bg-red-500/10 hover:bg-red-500/20 text-red-400 px-3 py-2 rounded-lg transition-all border border-red-500/20 hover:border-red-500/40 text-sm font-medium cursor-pointer"
            >
              <Trash2 className="w-4 h-4" />
              Clear
            </button>
          </div>
        </div>

        <div className="flex items-center gap-2 mb-3 overflow-x-auto pb-2 scrollbar-hide">
          {SOURCE_OPTIONS.map((source) => (
            <button
              key={source.value}
              onClick={() => onSourceChange(source.value)}
              className={`flex items-center gap-1.5 px-3 py-2 rounded-lg font-semibold text-sm transition-all whitespace-nowrap cursor-pointer ${
                selectedSource === source.value
                  ? 'bg-gradient-to-r from-blue-600 to-indigo-600 text-white shadow-lg shadow-blue-500/30'
                  : 'bg-gray-950/80 text-gray-500 hover:bg-gray-900 hover:text-gray-400 border border-gray-900'
              }`}
            >
              {getSourceIcon(source.value)}
              <span>{source.label}</span>
            </button>
          ))}
        </div>

        <div className="space-y-2.5">
          <div className="relative">
            <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4.5 h-4.5 text-gray-600" />
            <input
              type="text"
              value={searchTags}
              onChange={(e) => onSearchTagsChange(e.target.value.slice(0, maxInputLength))}
              onKeyPress={(e: React.KeyboardEvent<HTMLInputElement>) => e.key === 'Enter' && onSearch()}
              placeholder="anime, furina?, cat etc...."
              maxLength={maxInputLength}
              className="w-full bg-gray-950/80 backdrop-blur-sm border border-gray-900 rounded-lg pl-11 pr-4 py-3 text-gray-200 placeholder-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 transition-all text-sm"
            />
            <span className="absolute right-3.5 top-1/2 -translate-y-1/2 text-xs text-gray-800 font-mono">
              {searchTags.length}/{maxInputLength}
            </span>
          </div>

          <div className="flex gap-2.5">
            <input
              type="text"
              value={excludeTags}
              onChange={(e) => onExcludeTagsChange(e.target.value.slice(0, maxInputLength))}
              onKeyPress={(e: React.KeyboardEvent<HTMLInputElement>) => e.key === 'Enter' && onSearch()}
              placeholder="Exclude tags (optional)"
              maxLength={maxInputLength}
              className="flex-1 bg-gray-950/50 border border-gray-900 rounded-lg px-3.5 py-2.5 text-gray-200 placeholder-gray-700 focus:outline-none focus:ring-2 focus:ring-red-500/50 text-sm"
            />
            <button
              onClick={onSearch}
              disabled={loading}
              className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed px-6 py-2.5 rounded-lg font-bold transition-all flex items-center gap-2 shadow-lg shadow-blue-500/30 text-sm cursor-pointer"
            >
              {loading ? (
                <>
                  <Loader2 className="w-4.5 h-4.5 animate-spin" />
                  Yaweeee! loading...
                </>
              ) : (
                <>
                  <Search className="w-4.5 h-4.5" />
                  Search
                </>
              )}
            </button>
          </div>
        </div>

        {currentWallpaper && (
          <div className="mt-2.5 flex items-center gap-2 text-xs text-gray-700">
            <CheckCircle className="w-3.5 h-3.5 text-emerald-400" />
            Active: <span className="text-gray-600 font-medium">{currentWallpaper.split('/').pop()}</span>
          </div>
        )}
      </div>
    </div>
  );
};

export default ExpandedHeader;

