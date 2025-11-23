'use client';

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Image, Loader2 } from 'lucide-react';

import CustomTitleBar from './components/CustomTitleBar';
import CompactHeader from './components/headers/CompactHeader';
import ExpandedHeader from './components/headers/ExpandedHeader';
import ImageModal from './components/ImageModal';
import ImageCard from './components/ImageCard';

import {
  API_BASE_URL,
  DEFAULT_FETCH_COUNT,
  MAX_INPUT_LENGTH,
  UNLOAD_THRESHOLD,
} from './constants/wallpapers';
import { PicReImage, WallpaperItem, WallpaperSourceOption } from './types/wallpaper';

const ensureAbsoluteUrl = (value: string) => {
  if (!value) return '';
  if (value.startsWith('data:') || value.startsWith('blob:')) return value;
  if (value.startsWith('http://') || value.startsWith('https://')) return value;
  if (value.startsWith('//')) return `https:${value}`;
  if (value.startsWith('/')) return `https://${value.replace(/^\/+/, '')}`;
  return `https://${value}`;
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
    detailUrl: item?.detailUrl,
    original: item,
  };
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
  const [videoWallpaperState, setVideoWallpaperState] = useState({ isActive: false, videoPath: null as string | null, videoUrl: null as string | null });
  const [isTogglingLive, setIsTogglingLive] = useState(false);
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
  const isLoadingRef = useRef(false);

  const loadCacheInfo = useCallback(async () => {
    try {
      const result: any = await invoke('get_cache_size');
      if (result?.success) {
        setCacheInfo({ sizeMB: result.sizeMb, fileCount: result.fileCount });
      }
    } catch (error) {
      console.error('[ERROR] Cache load failed:', error);
    }
  }, []);

  const loadCurrentWallpaper = useCallback(async () => {
    try {
      const result: any = await invoke('get_current_wallpaper');
      if (result?.success && result?.message) {
        setCurrentWallpaper(result.message);
      }
    } catch (error) {
      console.error('[ERROR] Wallpaper get failed:', error);
    }
  }, []);

  const loadVideoWallpaperState = useCallback(async () => {
    try {
      const state: any = await invoke('get_video_wallpaper_status');
      setVideoWallpaperState({
        isActive: state.isActive || false,
        videoPath: state.videoPath || null,
        videoUrl: state.videoUrl || null,
      });
    } catch (error) {
      console.error('[ERROR] Video wallpaper state load failed:', error);
    }
  }, []);

  useEffect(() => {
    loadCacheInfo();
    loadCurrentWallpaper();
    loadVideoWallpaperState();
  }, [loadCacheInfo, loadCurrentWallpaper, loadVideoWallpaperState]);

  // Listen to video wallpaper state changes
  useEffect(() => {
    const unlisten = listen('video-wallpaper-changed', (event: any) => {
      const state = event.payload as any;
      setVideoWallpaperState({
        isActive: state.isActive || false,
        videoPath: state.videoPath || null,
        videoUrl: state.videoUrl || null,
      });
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleToggleLiveWallpaper = useCallback(async () => {
    if (isTogglingLive) return;
    
    setIsTogglingLive(true);
    try {
      const result: any = await invoke('toggle_video_wallpaper', { 
        enable: !videoWallpaperState.isActive 
      });
      if (result?.success) {
        await loadVideoWallpaperState();
      } else {
        alert('Failed: ' + (result?.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('[ERROR] Toggle failed:', error);
      alert('Error: ' + error);
    } finally {
      setIsTogglingLive(false);
    }
  }, [videoWallpaperState.isActive, isTogglingLive, loadVideoWallpaperState]);

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

  useEffect(() => {
    const handleVisibility = () => {
      const scrollTop = window.scrollY;
      const viewportHeight = window.innerHeight;
      const itemHeight = 300;
      const itemsPerRow = Math.floor(window.innerWidth / 400);

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
    handleVisibility();

    return () => {
      window.removeEventListener('scroll', throttledHandler);
      window.removeEventListener('resize', throttledHandler);
    };
  }, [wallpapers.length]);

  const fetchWallpapers = useCallback(
    async (pageNum: number = 1, append: boolean = false) => {
      if (isLoadingRef.current) {
        console.log('[SCROLL] Already loading, skipping...');
        return;
      }

      isLoadingRef.current = true;

      if (append) {
        setLoadingMore(true);
      } else {
        setLoading(true);
        setWallpapers([]);
        setPage(1);
        setHasMore(true);
      }

      console.log(`[FETCH] Starting fetch - Page: ${pageNum}, Append: ${append}`);

      try {
        const includeTags = searchTags.split(' ').filter((t) => t.trim());
        const excludeTagsArray = excludeTags.split(' ').filter((t) => t.trim());
        const queryString = includeTags.join(' ');

        if (selectedSource === 'picre') {
          const params: Record<string, string> = { compress: 'false' };
          if (includeTags.length > 0) params.in = includeTags.join(',');
          if (excludeTagsArray.length > 0) params.of = excludeTagsArray.join(',');

          const promises = Array(DEFAULT_FETCH_COUNT)
            .fill(null)
            .map(() =>
              fetch(`${API_BASE_URL}/image?${new URLSearchParams(params)}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', 'User-Agent': 'WallpaperApp/1.0' },
              }).then((res) => res.json())
            );

          const results = await Promise.allSettled(promises);
          const images = results
            .filter((r): r is PromiseFulfilledResult<PicReImage> => r.status === 'fulfilled' && r.value)
            .map((r) => r.value);

          console.log(`[FETCH] PicRe items: ${images.length}`);

          if (images.length === 0) {
            if (!append) alert('No images found. Try different tags.');
            setHasMore(false);
          } else {
            const normalized = images.map(normalizePicReImage);
            if (append) {
              setWallpapers((prev) => [...prev, ...normalized]);
            } else {
              setWallpapers(normalized);
            }
          }
          return;
        }

        let backendSources: string[] | undefined;
        if (selectedSource !== 'all') {
          backendSources = [selectedSource];
        } else {
          backendSources = ['wallhaven', 'zerochan', 'moewalls', 'wallpapers', 'wallpaperflare', 'motionbgs'];
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

        console.log(`[FETCH] Response - Items: ${response?.items?.length || 0}`);

        if (!response?.items || response.items.length === 0) {
          console.log('[FETCH] No items, stopping pagination');
          if (!append) alert('No wallpapers found. Try different filters.');
          setHasMore(false);
          return;
        }

        const normalized = (response.items as any[]).map(normalizeExternalItem).filter((item) => item.imageUrl);
        console.log(`[FETCH] Normalized: ${normalized.length}`);

        let newWallpapers: WallpaperItem[] = [];

        if (append) {
          setWallpapers((prev) => {
            const existingIds = new Set(prev.map((w) => w.id));
            newWallpapers = normalized.filter((item) => !existingIds.has(item.id));
            console.log(`[FETCH] New unique: ${newWallpapers.length}`);
            return [...prev, ...newWallpapers];
          });
        } else {
          newWallpapers = normalized;
          setWallpapers(newWallpapers);
        }

        if (newWallpapers.length < DEFAULT_FETCH_COUNT / 2) {
          console.log(`[FETCH] Got ${newWallpapers.length} items, stopping pagination`);
          setHasMore(false);
        }
      } catch (error) {
        console.error('[ERROR] Fetch failed:', error);
        if (!append) alert('Error fetching wallpapers: ' + error);
        setHasMore(false);
      } finally {
        setLoading(false);
        setLoadingMore(false);
        isLoadingRef.current = false;
        console.log('[FETCH] Completed');
      }
    },
    [excludeTags, searchTags, selectedSource]
  );

  const loadMoreWallpapers = useCallback(() => {
    if (!hasMore || isLoadingRef.current) {
      console.log(`[SCROLL] Skipping - hasMore: ${hasMore}, isLoading: ${isLoadingRef.current}`);
      return;
    }

    const nextPage = page + 1;
    console.log(`[SCROLL] Loading page ${nextPage}`);
    setPage(nextPage);
    fetchWallpapers(nextPage, true);
  }, [fetchWallpapers, hasMore, page]);

  useEffect(() => {
    // Cleanup previous observer
    if (observerRef.current) {
      console.log('[OBSERVER] Disconnecting previous observer');
      observerRef.current.disconnect();
    }

    // Only create observer if we have wallpapers and there's more to load
    if (wallpapers.length === 0) {
      console.log('[OBSERVER] No wallpapers yet, skipping observer setup');
      return;
    }

    // Create new observer
    console.log('[OBSERVER] Creating new intersection observer');
    observerRef.current = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          console.log(`[OBSERVER] Entry - isIntersecting: ${entry.isIntersecting}, target: ${entry.target.className}`);
          
          if (entry.isIntersecting) {
            console.log('[OBSERVER] Sentinel is intersecting!');
            if (hasMore && !isLoadingRef.current) {
              console.log('[OBSERVER] Conditions met - calling loadMoreWallpapers');
              loadMoreWallpapers();
            } else {
              console.log(`[OBSERVER] Conditions NOT met - hasMore: ${hasMore}, isLoading: ${isLoadingRef.current}`);
            }
          }
        });
      },
      {
        rootMargin: '500px',
        threshold: 0.01,
      }
    );

    // Observe the trigger element
    const currentSentinel = sentinelRef.current;
    if (currentSentinel) {
      console.log('[OBSERVER] Found sentinel element, observing it');
      observerRef.current.observe(currentSentinel);
    } else {
      console.log('[OBSERVER] ERROR: Sentinel element not found!');
    }

    // Cleanup
    return () => {
      if (observerRef.current) {
        console.log('[OBSERVER] Cleanup - disconnecting observer');
        observerRef.current.disconnect();
      }
    };
  }, [wallpapers.length, hasMore, loadMoreWallpapers]);

  const setAsWallpaper = useCallback(
    async (image: WallpaperItem, resolvedUrl?: string) => {
      if (image.type === 'video') {
        window.open(image.imageUrl, '_blank');
        return;
      }

      setSettingWallpaper(image.id);

      try {
        let finalUrl = resolvedUrl || image.imageUrl;

        if (!resolvedUrl && image.source === 'wallpaperflare' && image.detailUrl) {
          try {
            const result: any = await invoke('resolve_wallpaperflare_highres', { detailUrl: image.detailUrl });
            if (result?.success && result?.url) {
              finalUrl = result.url;
            }
          } catch (e) {
            console.warn('[WARN] Failed to resolve high-res, using thumbnail:', e);
          }
        }

        const result: any = await invoke('set_wallpaper', { imageUrl: finalUrl });

        if (result.success) {
          setCurrentWallpaper(finalUrl);
          setSelectedImage(null);
          await loadCacheInfo();
        } else {
          alert('Failed to set wallpaper: ' + result.error);
        }
      } catch (error) {
        console.error('[ERROR] Exception setting wallpaper:', error);
        alert('Error: ' + error);
      } finally {
        setSettingWallpaper(null);
      }
    },
    [loadCacheInfo]
  );

  const clearCache = useCallback(async () => {
    if (!confirm('Clear all cached wallpapers?')) return;

    try {
      const result: any = await invoke('clear_cache');
      if (result?.success) {
        alert(`Cache cleared: ${result.filesDeleted} files deleted`);
        await loadCacheInfo();
      }
    } catch (error) {
      console.error('[ERROR] Clear cache failed:', error);
      alert('Failed to clear cache: ' + error);
    }
  }, [loadCacheInfo]);

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

  const handleExpandHeader = useCallback(() => {
    setIsHeaderCompact(false);
    setShowExpandedHeader(true);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }, []);

  return (
    <div
      className="min-h-screen bg-black text-gray-100 relative pt-8"
      onContextMenu={(e: React.MouseEvent) => {
        e.preventDefault();
        return false;
      }}
    >
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

        * {
          scrollbar-width: thin;
          scrollbar-color: #3b82f6 #0a0a0a;
        }
        *::-webkit-scrollbar {
          width: 8px;
          height: 8px;
        }
        *::-webkit-scrollbar-track {
          background: #0a0a0a;
          border-radius: 4px;
        }
        *::-webkit-scrollbar-thumb {
          background: linear-gradient(180deg, #3b82f6, #6366f1);
          border-radius: 4px;
          border: 1px solid #1e293b;
        }
        *::-webkit-scrollbar-thumb:hover {
          background: linear-gradient(180deg, #2563eb, #4f46e5);
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

        [data-tauri-drag-region] {
          -webkit-app-region: drag;
          app-region: drag;
        }
        button {
          -webkit-app-region: no-drag !important;
          app-region: no-drag !important;
        }

        img {
          pointer-events: auto;
          -webkit-user-drag: none;
          -khtml-user-drag: none;
          -moz-user-drag: none;
          -o-user-drag: none;
          user-drag: none;
        }
      `}</style>

      {selectedImage && (
        <ImageModal
          image={selectedImage}
          onClose={() => setSelectedImage(null)}
          onSetWallpaper={(url) => setAsWallpaper(selectedImage, url)}
          isLoading={settingWallpaper === selectedImage.id}
        />
      )}

      <CompactHeader cacheInfo={cacheInfo} isHeaderCompact={isHeaderCompact} onExpand={handleExpandHeader} />

      <ExpandedHeader
        cacheInfo={cacheInfo}
        selectedSource={selectedSource}
        onSourceChange={setSelectedSource}
        searchTags={searchTags}
        onSearchTagsChange={setSearchTags}
        excludeTags={excludeTags}
        onExcludeTagsChange={setExcludeTags}
        maxInputLength={MAX_INPUT_LENGTH}
        loading={loading}
        onSearch={() => fetchWallpapers()}
        clearCache={clearCache}
        currentWallpaper={currentWallpaper}
        showExpandedHeader={showExpandedHeader}
        videoWallpaperState={videoWallpaperState}
        isTogglingLive={isTogglingLive}
        onToggleLive={handleToggleLiveWallpaper}
      />

      <div className="max-w-full mx-auto px-4 py-6 pt-14 relative z-10">
        {wallpapers.length === 0 && !loading && (
          <div className="text-center py-24 animate-fadeIn">
            <div className="inline-block p-5 bg-gray-950/50 backdrop-blur-sm rounded-2xl mb-4 border border-gray-900">
              <Image className="w-16 h-16 text-gray-800" />
            </div>
            <h2 className="text-xl font-bold text-gray-500 mb-2">Search for Wallpapers & Art</h2>
            <p className="text-gray-700 text-sm">Try anything ex: anime, landscape, rain, ecchi</p>
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
          <div 
            ref={sentinelRef} 
            className="py-8 text-center min-h-20 border-t border-gray-800"
            style={{
              background: 'rgba(30, 30, 30, 0.5)',
            }}
            onClick={() => console.log('[SENTINEL] Sentinel div clicked!')}
          >
            <p className="text-xs text-gray-600 mb-2">📍 Load more trigger zone</p>
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

