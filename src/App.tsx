'use client';
import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Search, Download, Trash2, HardDrive, Loader2, CheckCircle, Image, Play, X, ZoomIn, ZoomOut } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

type WallpaperSourceOption = 'all' | 'picre' | 'wallhaven' | 'zerochan' | 'wallpapers' | 'moewalls' | 'wallpaperflare';

interface PicReImage {
  file_url: string;
  md5: string;
  tags: string[];
  width: number;
  height: number;
  source: string;
  author: string;
  has_children: boolean;
  _id: number;
}

interface WallpaperItem {
  id: string;
  source: WallpaperSourceOption;
  title?: string;
  imageUrl: string;
  thumbnailUrl?: string;
  type?: 'image' | 'video';
  width?: number;
  height?: number;
  tags?: string[];
  metadata?: Record<string, unknown>;
  original?: any; // changed from unknown to any
}

const API_BASE_URL = 'https://pic.re';
const DEFAULT_FETCH_COUNT = 20;
const MAX_INPUT_LENGTH = 100;
const UNLOAD_THRESHOLD = 5; // Unload images 5 rows above/below viewport

const SOURCE_OPTIONS: { value: WallpaperSourceOption; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'wallhaven', label: 'Wallhaven' },
  { value: 'zerochan', label: 'Zerochan' },
  { value: 'wallpapers', label: 'Wallpapers' },
  { value: 'moewalls', label: 'Live2D' },
  { value: 'wallpaperflare', label: 'WPFlare' },
  { value: 'picre', label: 'pic.re' },
];

const GlobeIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="12" cy="12" r="10"/>
    <line x1="2" y1="12" x2="22" y2="12"/>
    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
  </svg>
);

const PaletteIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="12" cy="12" r="10"/>
    <circle cx="8" cy="10" r="1" fill="currentColor"/>
    <circle cx="12" cy="8" r="1" fill="currentColor"/>
    <circle cx="16" cy="10" r="1" fill="currentColor"/>
    <circle cx="14" cy="14" r="1" fill="currentColor"/>
    <path d="M12 22c1.5-2 3-3.5 3-6a3 3 0 0 0-6 0c0 2.5 1.5 4 3 6z"/>
  </svg>
);

const StarIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
  </svg>
);

const CameraIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
    <circle cx="12" cy="13" r="4"/>
  </svg>
);

const SparklesIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M12 3v18M3 12h18M5.6 5.6l12.8 12.8M18.4 5.6L5.6 18.4"/>
  </svg>
);

const FlameIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"/>
  </svg>
);

const BoltIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
  </svg>
);

const getSourceIcon = (source: WallpaperSourceOption) => {
  switch (source) {
    case 'all': return <GlobeIcon />;
    case 'wallhaven': return <PaletteIcon />;
    case 'zerochan': return <StarIcon />;
    case 'wallpapers': return <CameraIcon />;
    case 'moewalls': return <SparklesIcon />;
    case 'wallpaperflare': return <FlameIcon />;
    case 'picre': return <BoltIcon />;
    default: return <CameraIcon />;
  }
};

const LaxentaLogo = () => (
  <div className="relative w-8 h-8">
    <div className="absolute inset-0 rounded-full border-2 border-blue-500/40 animate-soundwave" />
    <div className="absolute inset-0 rounded-full border-2 border-blue-400/30 animate-soundwave" style={{ animationDelay: '0.5s' }} />
    <div className="absolute inset-0 rounded-full border-2 border-indigo-500/30 animate-soundwave" style={{ animationDelay: '1s' }} />
    <div className="absolute inset-0 rounded-full border-2 border-indigo-400/20 animate-soundwave" style={{ animationDelay: '1.5s' }} />
    
    <div className="relative w-8 h-8 rounded-full overflow-hidden border-2 border-blue-500/50 z-10">
      <img 
        src="/128x128.png" 
        alt="ColorWall uwugo" 
        className="w-full h-full object-cover"
        onError={(e: React.SyntheticEvent<HTMLImageElement>) => {
          e.currentTarget.style.display = 'none';
          e.currentTarget.parentElement!.style.background = 'linear-gradient(135deg, #000, #000)';
        }}
      />
    </div>
  </div>
);

const ImageModal = ({ 
  image, 
  onClose, 
  onSetWallpaper, 
  isLoading 
}: { 
  image: WallpaperItem; 
  onClose: () => void; 
  onSetWallpaper: () => void;
  isLoading: boolean;
}) => {
  const [zoom, setZoom] = useState(1);
  const [imgLoaded, setImgLoaded] = useState(false);
  const [highResUrl, setHighResUrl] = useState<string>(image.imageUrl);

  useEffect(() => {
    // fetch high-res if not already present and source is wallpaperflare
    if (image.source === 'wallpaperflare' && image.thumbnailUrl && image.imageUrl === image.thumbnailUrl) {
      (async () => {
        try {
          const result: any = await invoke('resolve_wallpaperflare_highres', { detailUrl: image.original?.detailUrl ?? image.imageUrl });
          if (result?.success && result?.url) {
            setHighResUrl(result.url);
          }
        } catch (e) {
          // fallback thumbnail
        }
      })();
    }
  }, [image]);

  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleEsc);
    return () => window.removeEventListener('keydown', handleEsc);
  }, [onClose]);

  return (
    <div 
      className="fixed inset-0 z-[9999] bg-black/98 backdrop-blur-xl flex items-center justify-center p-4"
      onClick={onClose}
    >
      <button
        onClick={onClose}
        className="absolute top-4 right-4 z-10 p-2 bg-black/80 hover:bg-gray-900 rounded-full transition-colors cursor-pointer border border-gray-800"
      >
        <X className="w-6 h-6 text-gray-400" />
      </button>

      <div className="absolute top-4 left-4 z-10 flex gap-2">
        <button
          onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
            e.stopPropagation();
            setZoom(Math.min(zoom + 0.25, 3));
          }}
          className="p-2 bg-black/80 hover:bg-gray-900 rounded-full transition-colors cursor-pointer border border-gray-800"
        >
          <ZoomIn className="w-5 h-5 text-gray-400" />
        </button>
        <button
          onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
            e.stopPropagation();
            setZoom(Math.max(zoom - 0.25, 0.5));
          }}
          className="p-2 bg-black/80 hover:bg-gray-900 rounded-full transition-colors cursor-pointer border border-gray-800"
        >
          <ZoomOut className="w-5 h-5 text-gray-400" />
        </button>
        <button
          onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
            e.stopPropagation();
            setZoom(1);
          }}
          className="px-3 py-2 bg-black/80 hover:bg-gray-900 rounded-full transition-colors text-xs text-gray-400 cursor-pointer border border-gray-800"
        >
          Reset
        </button>
      </div>

      <div className="relative max-w-7xl max-h-[90vh] overflow-auto" onClick={(e) => e.stopPropagation()}>
        {!imgLoaded && (
          <div className="absolute inset-0 flex items-center justify-center">
            <Loader2 className="w-12 h-12 animate-spin text-blue-500" />
          </div>
        )}
        
        <img
          src={highResUrl}
          alt={image.title || 'Wallpaper'}
          className="max-w-full max-h-[90vh] object-contain transition-all duration-300"
          style={{ transform: `scale(${zoom})` }}
          onLoad={() => setImgLoaded(true)}
        />

        <div className="absolute bottom-4 left-1/2 -translate-x-1/2 flex gap-3">
          <a
            href={image.imageUrl}
            download
            onClick={(e: React.MouseEvent<HTMLAnchorElement>) => e.stopPropagation()}
            className="flex items-center gap-2 bg-black/90 hover:bg-gray-900 text-white px-6 py-3 rounded-full transition-all font-medium shadow-xl cursor-pointer border border-gray-800"
          >
            <Download className="w-5 h-5" />
            Download
          </a>
          
          {image.type !== 'video' && (
            <button
              onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                e.stopPropagation();
                onSetWallpaper();
              }}
              disabled={isLoading}
              className="flex items-center gap-2 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 disabled:opacity-50 text-white px-6 py-3 rounded-full transition-all font-medium shadow-xl shadow-blue-500/30 cursor-pointer"
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin" />
                  Setting...
                </>
              ) : (
                <>
                  <CheckCircle className="w-5 h-5" />
                  Set as Wallpaper
                </>
              )}
            </button>
          )}
        </div>

        <div className="absolute top-4 left-1/2 -translate-x-1/2 bg-black/90 backdrop-blur-md px-4 py-2 rounded-full border border-gray-800">
          <p className="text-sm text-gray-300 font-medium">{image.title || 'Untitled'}</p>
        </div>
      </div>
    </div>
  );
};

const ImageCard = ({ 
  image, 
  onSelect,
  isVisible 
}: { 
  image: WallpaperItem; 
  onSelect: () => void;
  isVisible: boolean;
}) => {
  const [imgLoaded, setImgLoaded] = useState(false);
  const [imgError, setImgError] = useState(false);
  const [shouldLoad, setShouldLoad] = useState(false);
  const imgRef = useRef<HTMLImageElement>(null);
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isVisible) {
      setShouldLoad(false);
      setImgLoaded(false);
      if (imgRef.current) {
        imgRef.current.src = '';
      }
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setShouldLoad(true);
        }
      },
      { rootMargin: '200px' }
    );

    if (cardRef.current) {
      observer.observe(cardRef.current);
    }

    return () => observer.disconnect();
  }, [isVisible]);

  useEffect(() => {
    if (shouldLoad && imgRef.current && !imgRef.current.src) {
      const src = image.thumbnailUrl || image.imageUrl;
      imgRef.current.src = src;
    }
  }, [shouldLoad, image]);

  return (
    <div
      ref={cardRef}
      className="group relative bg-black/70 rounded-xl overflow-hidden border border-gray-900/50 hover:border-blue-500/30 transition-all duration-300 cursor-pointer shadow-lg hover:shadow-xl hover:shadow-blue-500/10 backdrop-blur-sm animate-fadeIn break-inside-avoid mb-4"
      onClick={onSelect}
      style={{ height: 'auto' }}
    >
      <div
        className="relative overflow-hidden bg-black flex items-center justify-center"
        style={{ height: 'auto' }}
      >
        {!imgLoaded && !imgError && shouldLoad && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="w-6 h-6 border-2 border-blue-500/20 border-t-blue-500 rounded-full animate-spin" />
          </div>
        )}
        
        {shouldLoad ? (
          <img
            ref={imgRef}
            alt={image.title ?? `Wallpaper ${image.id}`}
            className={`w-full h-auto object-contain group-hover:scale-105 transition-all duration-500 ${imgLoaded ? 'opacity-100' : 'opacity-0'}`}
            loading="lazy"
            onLoad={() => setImgLoaded(true)}
            onError={() => setImgError(true)}
          />
        ) : (
          <div className="absolute inset-0 bg-gray-950 flex items-center justify-center">
            <div className="w-6 h-6 border-2 border-gray-800 border-t-gray-700 rounded-full animate-spin" />
          </div>
        )}
        
        {imgError && (
          <div className="absolute inset-0 flex items-center justify-center text-gray-700">
            <div className="text-center">
              <Image className="w-6 h-6 mx-auto mb-2 opacity-50" />
              <span className="text-xs">Failed to load</span>
            </div>
          </div>
        )}
        
        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/10 to-transparent opacity-0 group-hover:opacity-100 transition-all duration-300 flex items-center justify-center">
          {image.type === 'video' ? (
            <div className="flex items-center gap-2 bg-emerald-500/90 px-4 py-2 rounded-full text-sm font-semibold">
              <Play className="w-4 h-4" />
              View Live2D
            </div>
          ) : (
            <div className="flex items-center gap-2 bg-blue-500/90 px-4 py-2 rounded-full text-sm font-semibold">
              <ZoomIn className="w-4 h-4" />
              View Full Size
            </div>
          )}
        </div>
      </div>

      <div className="p-3 bg-black/90 backdrop-blur-sm">
        <div className="flex items-center justify-between mb-1.5">
          <span className="inline-flex items-center gap-1.5 text-xs font-bold uppercase tracking-wider text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded">
            {getSourceIcon(image.source)}
            <span>{image.source}</span>
          </span>
          {image.width && image.height && (
            <span className="text-xs text-gray-600 font-mono">
              {image.width}×{image.height}
            </span>
          )}
        </div>
        <p className="text-xs text-gray-400 truncate" title={image.title}>
          {image.title || 'Untitled'}
        </p>
      </div>
    </div>
  );
};

// fk tauri, i had to debug this for 30 minutes to figure out tauri was silently blocking it :sob: 
// while i tried to find z indexes or overlaps or cursor pointers lmao
const CustomTitleBar = () => {
  const [isMaximized, setIsMaximized] = useState(false);

  const minimize = async () => {
    console.log('[DEBUG] Minimize button clicked!');
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      console.log('[DEBUG] Window API imported');
      const window = getCurrentWindow();
      console.log('[DEBUG] Got current window:', window);
      await window.minimize();
      console.log('[SUCCESS] Window minimized');
    } catch (err) {
      console.error('[ERROR] Minimize failed:', err);
      alert('Minimize failed: ' + err);
    }
  };
  const toggleMaximize = async () => {
    console.log('[DEBUG] Maximize button clicked!');
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      console.log('[DEBUG] Window API imported from tauri win api, what about linux? it might not work on some sorry man if u see this');
      const appWindow = getCurrentWindow();
      console.log('[DEBUG] Got current window:', appWindow);
      const currentMaximized = await appWindow.isMaximized();
      console.log('[DEBUG] Current maximized state:', currentMaximized);
      await appWindow.toggleMaximize();
      const newMaximized = await appWindow.isMaximized();
      console.log('[DEBUG] New maximized state:', newMaximized);
      setIsMaximized(newMaximized);
      console.log('[SUCCESS] Window toggled');
    } catch (err) {
      console.error('[ERROR] Maximize failed:', err);
      alert('Maximize failed: ' + err);
    }
  };
  const close = async () => {
    console.log('[DEBUG] Close button clicked ;=;');
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      console.log('[DEBUG] Window API imported');
      const window = getCurrentWindow();
      console.log('[DEBUG] Got current window:', window);
      await window.close();
      console.log('[SUCCESS] Window closed');
    } catch (err) {
      console.error('[ERROR] Close failed:', err);
      alert('Close failed: ' + err);
    }
  };
  return (
    <div className="fixed top-0 left-0 right-0 h-8 bg-black/98 backdrop-blur-xl border-b border-gray-900/50 flex items-center justify-between px-3 z-[99999] select-none">
      <div 
        data-tauri-drag-region 
        className="flex items-center gap-2 flex-1 h-full"
      >
        <div className="w-5 h-5 relative pointer-events-none">
          <img src="/128x128.png" alt="" className="w-full h-full object-contain" />
        </div>
        <span className="text-xs font-semibold text-gray-500">ColorWall</span>
      </div>
      
      {/* Window controls */}
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

export default function WallpaperEngine() {
  const [wallpapers, setWallpapers] = useState<WallpaperItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [searchTags, setSearchTags] = useState('demon slayer');
  const [excludeTags, setExcludeTags] = useState('');
  const [cacheInfo, setCacheInfo] = useState({ sizeMB: '0', fileCount: 0 });
  const [currentWallpaper, setCurrentWallpaper] = useState('');
  const [settingWallpaper, setSettingWallpaper] = useState<string | null>(null);
  const [selectedSource, setSelectedSource] = useState<WallpaperSourceOption>('all');
  const [selectedImage, setSelectedImage] = useState<WallpaperItem | null>(null);
  const [isHeaderCompact, setIsHeaderCompact] = useState(false);
  const [showExpandedHeader, setShowExpandedHeader] = useState(true);
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);
  const [visibleRange, setVisibleRange] = useState({ start: 0, end: 20 });
  const lastScrollY = useRef(0);
  const observerRef = useRef<IntersectionObserver | null>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    console.log('[INFO] Wallpaper Engine initialized');
    loadCacheInfo();
    loadCurrentWallpaper();
  }, []);

  useEffect(() => {
    let ticking = false;
    const handleScroll = () => {
      if (!ticking) {
        window.requestAnimationFrame(() => {
          const currentScrollY = window.scrollY;
          
          if (currentScrollY > 100 && currentScrollY > lastScrollY.current) {
            setIsHeaderCompact(true);
            setShowExpandedHeader(false);
          } else if (currentScrollY < lastScrollY.current || currentScrollY < 50) {
            setIsHeaderCompact(false);
            setShowExpandedHeader(true);
          }
          
          lastScrollY.current = currentScrollY;
          ticking = false;
        });
        ticking = true;
      }
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  // Calculate visible range based on scroll position
  useEffect(() => {
    const handleVisibility = () => {
      const scrollTop = window.scrollY;
      const viewportHeight = window.innerHeight;
      const itemHeight = 300; 
      const itemsPerRow = Math.floor(window.innerWidth / 400); // approx items per row
      
      const startRow = Math.max(0, Math.floor((scrollTop - viewportHeight) / itemHeight) - UNLOAD_THRESHOLD);
      const endRow = Math.ceil((scrollTop + viewportHeight * 2) / itemHeight) + UNLOAD_THRESHOLD;
      
      const start = startRow * itemsPerRow;
      const end = endRow * itemsPerRow;
      
      setVisibleRange({ start, end });
    };

    let ticking = false;
    const throttledHandler = () => {
      if (!ticking) {
        window.requestAnimationFrame(() => {
          handleVisibility();
          ticking = false;
        });
        ticking = true;
      }
    };

    window.addEventListener('scroll', throttledHandler, { passive: true });
    window.addEventListener('resize', throttledHandler, { passive: true });
    handleVisibility(); // initial calc

    return () => {
      window.removeEventListener('scroll', throttledHandler);
      window.removeEventListener('resize', throttledHandler);
    };
  }, [wallpapers.length]);

  const loadMoreWallpapers = useCallback(() => {
    if (!hasMore || loadingMore) return;
    const nextPage = page + 1;
    setPage(nextPage);
    fetchWallpapers(nextPage, true);
  }, [page, hasMore, loadingMore, wallpapers]);

  // scroll observer
  useEffect(() => {
    if (!hasMore || loadingMore) return;

    observerRef.current = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !loadingMore) {
          loadMoreWallpapers();
        }
      },
      { rootMargin: '500px' }
    );

    if (sentinelRef.current) {
      observerRef.current.observe(sentinelRef.current);
    }



    return () => observerRef.current?.disconnect();
  }, [hasMore, loadingMore, loadMoreWallpapers]);

  const loadCacheInfo = async () => {
    try {
      const result: any = await invoke('get_cache_size');
      if (result.success) {
        setCacheInfo({ sizeMB: result.sizeMb, fileCount: result.fileCount });
        // console.log(`[SUCCESS] Cache: ${result.sizeMb}MB, ${result.fileCount} files`);
      }
    } catch (error) {
      console.error('[ERROR] Cache load failed:', error);
    }
  };

  const loadCurrentWallpaper = async () => {
    try {
      const result: any = await invoke('get_current_wallpaper');
      if (result.success && result.message) {
        setCurrentWallpaper(result.message);
        // console.log(`[INFO] Current wallpaper: ${result.message.split('/').pop()}`);
      }
    } catch (error) {
      console.error('[ERROR] Wallpaper get failed:', error);
    }
  };

  const normalizePicReImage = (image: PicReImage): WallpaperItem => {
    const fullUrl = image.file_url.startsWith('http') ? image.file_url : `https://${image.file_url}`;
    return {
      id: `picre-${image._id}-${image.md5}`,
      source: 'picre',
      title: image.source || `Wallpaper ${image._id}`,
      imageUrl: fullUrl,
      thumbnailUrl: fullUrl,
      type: 'image',
      width: image.width,
      height: image.height,
      tags: image.tags,
      metadata: { author: image.author, hasChildren: image.has_children },
      original: image,
    };
  };

  const ensureAbsoluteUrl = (value: string) => {
    if (!value) return '';
    if (value.startsWith('data:') || value.startsWith('blob:')) return value;
    if (value.startsWith('http://') || value.startsWith('https://')) return value;
    if (value.startsWith('//')) return `https:${value}`;
    if (value.startsWith('/')) return `https://${value.replace(/^\/+/, '')}`;
    return `https://${value}`;
  };

  const normalizeExternalItem = (item: any): WallpaperItem => {
    const source = (item?.source ?? 'wallhaven') as WallpaperSourceOption;
    return {
      id: item?.id ?? `${source}-${Math.random().toString(36).slice(2, 10)}`,
      source,
      title: item?.title ?? item?.metadata?.title ?? item?.id ?? source.toUpperCase(),
      imageUrl: ensureAbsoluteUrl(item?.imageUrl ?? item?.thumbnailUrl ?? ''),
      thumbnailUrl: ensureAbsoluteUrl(item?.thumbnailUrl ?? item?.imageUrl ?? ''),
      type: item?.type === 'video' ? 'video' : 'image',
      width: item?.width,
      height: item?.height,
      tags: Array.isArray(item?.tags) ? item.tags : [],
      metadata: item?.metadata ?? {},
      original: item,
    };
  };

  const fetchWallpapers = async (pageNum: number = 1, append: boolean = false) => {
    if (append) {
      setLoadingMore(true);
    } else {
      setLoading(true);
      setWallpapers([]);
      setPage(1);
      setHasMore(true);
    }

    try {
      const includeTags = searchTags.split(' ').filter(t => t.trim());
      const excludeTagsArray = excludeTags.split(' ').filter(t => t.trim());
      const queryString = includeTags.join(' ');
  
      if (selectedSource === 'picre') {
        console.log(`[INFO] Fetching from pic.re page ${pageNum}: [${includeTags.join(', ')}]`);
        const params: Record<string, string> = { compress: 'false' };
        if (includeTags.length > 0) params.in = includeTags.join(',');
        if (excludeTagsArray.length > 0) params.of = excludeTagsArray.join(',');
  
        const promises = Array(DEFAULT_FETCH_COUNT).fill(null).map(() => 
          fetch(`${API_BASE_URL}/image?${new URLSearchParams(params)}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'User-Agent': 'WallpaperApp/1.0' }
          }).then(res => res.json())
        );
  
        const results = await Promise.allSettled(promises);
        const images = results
          .filter((r): r is PromiseFulfilledResult<PicReImage> => r.status === 'fulfilled' && r.value)
          .map(r => r.value);
        
        if (images.length === 0) {
          console.warn('[WARN] No images found');
          if (!append) alert('No images found. Try different tags.');
          setHasMore(false);
        } else {
          const normalized = images.map(normalizePicReImage);
          if (append) {
            setWallpapers(prev => [...prev, ...normalized]);
          } else {
            setWallpapers(normalized);
          }
          console.log(`[SUCCESS] Loaded ${images.length} wallpapers`);
        }
        return;
      }
  
      console.log(`[INFO] Fetching from ${selectedSource} page ${pageNum}: "${queryString}"`);
      
      let backendSources: string[] | undefined;
      if (selectedSource !== 'all') {
        backendSources = [selectedSource];
      } else {
        backendSources = ['wallhaven', 'zerochan', 'moewalls', 'wallpapers', 'wallpaperflare'];
      }
      
      const response: any = await invoke('search_wallpapers', {
        query: queryString || 'anime',
        sources: backendSources,
        limitPerSource: DEFAULT_FETCH_COUNT,
        randomize: true,
        page: pageNum,
        purity: '100',
        aiArt: false,
      });

    if (!response?.items || response.items.length === 0) {
      console.warn('[WARN] No items returned');
      if (!append) alert('No wallpapers found. Try different filters.');
      setHasMore(false);
      return;
    }

    const normalized = (response.items as any[])
      .map(normalizeExternalItem)
      .filter(item => item.imageUrl);

    // dupe by id
    let newWallpapers: WallpaperItem[] = [];
    if (append) {
      const existingIds = new Set(wallpapers.map(w => w.id));
      newWallpapers = normalized.filter(item => !existingIds.has(item.id));
      setWallpapers(prev => [...prev, ...newWallpapers]);
    } else {
      newWallpapers = normalized;
      setWallpapers(newWallpapers);
    }

    // if we got less than expected (ex we ask 20 it gave 19, it means the prov is out of wallpapers for the tag), stop loading more
    if (newWallpapers.length < DEFAULT_FETCH_COUNT / 2) {
      setHasMore(false);
    }
      
    } catch (error) {
      console.error('[ERROR] Fetch failed:', error);
      if (!append) alert('Error fetching wallpapers: ' + error);
      setHasMore(false);
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  };

  const setAsWallpaper = async (image: WallpaperItem) => {
    if (image.type === 'video') {
      console.log('[INFO] Opening Live2D video');
      window.open(image.imageUrl, '_blank');
      return;
    }

    setSettingWallpaper(image.id);
    
    try {
      console.log(`[INFO] Setting wallpaper: ${image.id}...`);
      
      const result: any = await invoke('set_wallpaper', { 
        imageUrl: image.imageUrl 
      });
      
      if (result.success) {
        setCurrentWallpaper(image.imageUrl);
        console.log('[SUCCESS] Wallpaper set! 🎨');
        setSelectedImage(null);
        await loadCacheInfo();
      } else {
        console.error('[ERROR] Set wallpaper failed:', result.error);
        alert('Failed to set wallpaper: ' + result.error);
      }
    } catch (error) {
      console.error('[ERROR]', error);
      alert('Error: ' + error);
    } finally {
      setSettingWallpaper(null);
    }
  };

  const clearCache = async () => {
    if (!confirm('Clear all cached wallpapers?')) return;
    
    try {
      const result: any = await invoke('clear_cache');
      if (result.success) {
        console.log(`[SUCCESS] Cleared ${result.filesDeleted} files`);
        alert(`Cache cleared: ${result.filesDeleted} files deleted`);
        await loadCacheInfo();
      }
    } catch (error) {
      console.error('[ERROR] Clear cache failed:', error);
      alert('Failed to clear cache: ' + error);
    }
  };

  // F11 toggle
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'F11') {
        e.preventDefault();
        (async () => {
          try {
            const { getCurrentWindow } = await import('@tauri-apps/api/window');
            const appWindow = getCurrentWindow();
            const isFullscreen = await appWindow.isFullscreen();
            await appWindow.setFullscreen(!isFullscreen);
          } catch (err) {
            console.error('Fullscreen toggle failed:', err);
          }
        })();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);
//  yes i have css here
  return (
    <div className="min-h-screen bg-black text-gray-100 relative pt-8">
      <CustomTitleBar />
      <style>{`
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(10px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .animate-fadeIn {
          animation: fadeIn 0.4s ease-out forwards;
        }
        @keyframes soundwave {
          0% {
            transform: scale(1);
            opacity: 0.6;
          }
          100% {
            transform: scale(2.5);
            opacity: 0;
          }
        }
        .animate-soundwave {
          animation: soundwave 2s ease-out infinite;
        }
        .scrollbar-hide {
          -ms-overflow-style: none;
          scrollbar-width: none;
        }
        .scrollbar-hide::-webkit-scrollbar {
          display: none;
        }
        
        button, a, input, select, textarea {
          -webkit-app-region: no-drag;
        }
        .sticky {
          -webkit-app-region: no-drag;
        }
        
        * {
          -webkit-user-select: none;
          -moz-user-select: none;
          -ms-user-select: none;
          user-select: none;
        }
        
        input, textarea {
          -webkit-user-select: text;
          -moz-user-select: text;
          -ms-user-select: text;
          user-select: text;
        }
        /* --- Drag region fixes --- */
        [data-tauri-drag-region] {
          -webkit-app-region: drag;
          app-region: drag;
        }
        button {
          -webkit-app-region: no-drag !important;
          app-region: no-drag !important;
        }
      `}</style>

      {selectedImage && (
        <ImageModal
          image={selectedImage}
          onClose={() => setSelectedImage(null)}
          onSetWallpaper={() => setAsWallpaper(selectedImage)}
          isLoading={settingWallpaper === selectedImage.id}
        />
      )}
      <div 
        className={`fixed top-8 left-0 right-0 z-50 transition-all duration-500 ease-out ${
          isHeaderCompact 
            ? 'translate-y-0 opacity-100' 
            : '-translate-y-full opacity-0 pointer-events-none'
        }`}
      >
        <div className="bg-black/98 backdrop-blur-xl border-b border-gray-900 shadow-2xl">
          <div className="max-w-7xl mx-auto px-4 py-2.5 flex items-center justify-between">
            <button
              onClick={() => {
                setIsHeaderCompact(false);
                setShowExpandedHeader(true);
                window.scrollTo({ top: 0, behavior: 'smooth' });
              }}
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
      <div 
        className={`bg-black/98 backdrop-blur-xl border-b border-gray-900 sticky top-8 z-40 shadow-2xl transition-all duration-500 ease-out ${
          showExpandedHeader ? 'translate-y-0 opacity-100' : '-translate-y-full opacity-0 pointer-events-none'
        }`}
      >
        <div className="max-w-7xl mx-auto px-4 py-4 relative z-10">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2.5">
              <LaxentaLogo />
              <div>
                <h1 className="text-xl font-bold bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
                  ColorWall
                </h1>
                <p className="text-xs text-gray-600">
                  Laxenta Inc
                </p>
                <p className="text-xs text-gray-700">
                  https://laxenta.tech
                </p>
              </div>
            </div>
            
            <div className="flex items-center gap-2.5">
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
                onClick={() => setSelectedSource(source.value)}
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
                onChange={(e) => setSearchTags(e.target.value.slice(0, MAX_INPUT_LENGTH))}
                onKeyPress={(e: React.KeyboardEvent<HTMLInputElement>) => e.key === 'Enter' && fetchWallpapers()}
                placeholder="anime..."
                maxLength={MAX_INPUT_LENGTH}
                className="w-full bg-gray-950/80 backdrop-blur-sm border border-gray-900 rounded-lg pl-11 pr-4 py-3 text-gray-200 placeholder-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 transition-all text-sm"
              />
              <span className="absolute right-3.5 top-1/2 -translate-y-1/2 text-xs text-gray-800 font-mono">
                {searchTags.length}/{MAX_INPUT_LENGTH}
              </span>
            </div>
            
            <div className="flex gap-2.5">
              <input
                type="text"
                value={excludeTags}
                onChange={(e) => setExcludeTags(e.target.value.slice(0, MAX_INPUT_LENGTH))}
                onKeyPress={(e: React.KeyboardEvent<HTMLInputElement>) => e.key === 'Enter' && fetchWallpapers()}
                placeholder="Exclude tags (optional)"
                maxLength={MAX_INPUT_LENGTH}
                className="flex-1 bg-gray-950/50 border border-gray-900 rounded-lg px-3.5 py-2.5 text-gray-200 placeholder-gray-700 focus:outline-none focus:ring-2 focus:ring-red-500/50 text-sm"
              />
              <button
                onClick={() => fetchWallpapers()}
                disabled={loading}
                className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed px-6 py-2.5 rounded-lg font-bold transition-all flex items-center gap-2 shadow-lg shadow-blue-500/30 text-sm cursor-pointer"
              >
                {loading ? (
                  <>
                    <Loader2 className="w-4.5 h-4.5 animate-spin" />
                    Loading...
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

      <div className="max-w-full mx-auto px-4 py-6 pt-14 relative z-10">
        {wallpapers.length === 0 && !loading && (
          <div className="text-center py-24 animate-fadeIn">
            <div className="inline-block p-5 bg-gray-950/50 backdrop-blur-sm rounded-2xl mb-4 border border-gray-900">
              <Image className="w-16 h-16 text-gray-800" />
            </div>
            <h2 className="text-xl font-bold text-gray-500 mb-2">Search for wallpapers</h2>
            <p className="text-gray-700 text-sm">Try: anime, landscape, rain, nature</p>
          </div>
        )}

        {loading && (
          <div className="text-center py-24 animate-fadeIn">
            <Loader2 className="w-14 h-14 mx-auto text-blue-500 animate-spin mb-4" strokeWidth={1.5} />
            <p className="text-lg font-semibold text-gray-600">mhmm...</p>
          </div>
        )}

        <div className="columns-1 sm:columns-2 lg:columns-3 gap-4 space-y-4">
          {wallpapers.map((image, index) => (
            <ImageCard
              key={image.id}
              image={image}
              onSelect={() => setSelectedImage(image)}
              isVisible={index >= visibleRange.start && index <= visibleRange.end}
            />
          ))}
        </div>

        {wallpapers.length > 0 && hasMore && (
          <div ref={sentinelRef} className="py-8 text-center">
            {loadingMore && (
              <div className="flex flex-col items-center gap-3 animate-fadeIn">
                <Loader2 className="w-10 h-10 text-blue-500 animate-spin" strokeWidth={1.5} />
                <p className="text-sm font-medium text-gray-600">Loading more...</p>
              </div>
            )}
          </div>
        )}

        {!hasMore && wallpapers.length > 0 && (
          <div className="text-center py-12 animate-fadeIn">
            <p className="text-gray-600 font-medium">This filter has No more wallpapers to load</p>
            <p className="text-xs text-gray-700 mt-1">Contribute on github for more improvements and providers ;c</p>
          </div>
        )}
      </div>
    </div>
  );
}