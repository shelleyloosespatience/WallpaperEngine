import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { Search, ArrowLeft, ArrowLeftRight } from 'lucide-react';
import WelcomeModal from '../components/WelcomeModal';
import ImageModal from '../components/ImageModal';
import { LoadingSpinner } from '../components/LoadingState';
import { WallpaperItem } from '../types/wallpaper';

interface StorePageProps {
    selectedSource: string;
    filterType?: 'all' | 'live' | 'static';
    isDirectNavigation?: boolean;
    onBack?: () => void;
}

export default function StorePage({ selectedSource, filterType = 'all', isDirectNavigation = false, onBack }: StorePageProps) {
    const [wallpapers, setWallpapers] = React.useState<WallpaperItem[]>([]);
    const [loading, setLoading] = React.useState(false);
    const [searchQuery, setSearchQuery] = React.useState('');
    const [hasMore, setHasMore] = React.useState(true);
    const [loadingMore, setLoadingMore] = React.useState(false);
    const [selectedImage, setSelectedImage] = React.useState<WallpaperItem | null>(null);
    const [settingWallpaper, setSettingWallpaper] = React.useState<string | null>(null);
    const [showWelcome, setShowWelcome] = React.useState(false);
    const [currentType, setCurrentType] = React.useState<'static' | 'live' | 'all'>(filterType as any);
    const pageRef = React.useRef(1);

    React.useEffect(() => {
        if (isDirectNavigation) {
            setShowWelcome(true);
        }
    }, [isDirectNavigation]);

    const searchWallpapers = React.useCallback(
        async (pageNum: number = 1, append: boolean = false) => {
            if (append) {
                setLoadingMore(true);
            } else {
                setLoading(true);
                setWallpapers([]);
            }

            try {
                let sourcesToUse: string[];

                if (currentType === 'live') {
                    sourcesToUse = ['motionbgs'];
                } else if (currentType === 'all') {
                    // init load: fast results from wallpaperflare only
                    // then uh the next loads: we do all sources
                    if (!append) {
                        sourcesToUse = ['wallpaperflare', 'motionbgs'];
                    } else {
                        sourcesToUse = ['wallhaven', 'moewalls', 'wallpapers', 'wallpaperflare', 'motionbgs'];
                    }
                } else {
                    // static wallpapers
                    if (selectedSource !== 'all') {
                        sourcesToUse = [selectedSource];
                    } else {
                        // init load: fast results from wallpaperflare only (hella fast!)
                        // sub loads: all static sources for diversity
                        if (!append) {
                            sourcesToUse = ['wallpaperflare'];
                        } else {
                            sourcesToUse = ['wallhaven', 'moewalls', 'wallpapers', 'wallpaperflare'];
                        }
                    }
                }

                const result: any = await invoke('search_wallpapers', {
                    query: searchQuery || 'anime',
                    sources: sourcesToUse,
                    limitPerSource: 30,
                    randomize: true,
                    page: pageNum,
                    purity: '100',
                    aiArt: false,
                });

                if (result.success && result.items) {
                    const normalized = result.items.map((item: any): WallpaperItem => ({
                        id: item.id,
                        source: item.source,
                        title: item.title,
                        imageUrl: item.imageUrl || item.image_url,
                        thumbnailUrl: item.thumbnailUrl || item.thumbnail_url || item.imageUrl || item.image_url,
                        type: item.type === 'video' || item.media_type === 'video' ? 'video' : 'image',
                        width: item.width,
                        height: item.height,
                        tags: item.tags,
                        detailUrl: item.detailUrl || item.detail_url,
                        metadata: item.metadata,
                        original: item,
                    }));

                    // Dont filter when type is 'all' - show everything mixed!
                    let filtered = normalized;
                    if (currentType === 'live') {
                        filtered = normalized.filter((item: WallpaperItem) => item.type === 'video');
                    } else if (currentType === 'static') {
                        filtered = normalized.filter((item: WallpaperItem) => item.type === 'image');
                    }

                    if (append) {
                        setWallpapers((prev) => [...prev, ...filtered]);
                    } else {
                        setWallpapers(filtered);
                    }

                    setHasMore(filtered.length >= 12);
                } else {
                    setHasMore(false);
                }
            } catch (error) {
                console.error('Search failed:', error);
                setHasMore(false);
            } finally {
                setLoading(false);
                setLoadingMore(false);
            }
        },
        [searchQuery, selectedSource, currentType]
    );

    React.useEffect(() => {
        searchWallpapers(1, false);
    }, [currentType]);

    React.useEffect(() => {
        const handleScroll = () => {
            if (
                window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 600 &&
                !loadingMore &&
                !loading &&
                hasMore
            ) {
                const nextPage = pageRef.current + 1;
                pageRef.current = nextPage;
                searchWallpapers(nextPage, true);
            }
        };

        window.addEventListener('scroll', handleScroll, { passive: true });
        return () => window.removeEventListener('scroll', handleScroll);
    }, [loadingMore, loading, hasMore, searchWallpapers]);

    const handleSearch = () => {
        pageRef.current = 1;
        setHasMore(true);
        searchWallpapers(1, false);
    };

    const handleWelcomeChoice = (type: 'static' | 'live' | 'all') => {
        setCurrentType(type);
        setShowWelcome(false);
    };

    const handleSetWallpaper = async (url: string) => {
        if (!selectedImage || settingWallpaper) return;

        setSettingWallpaper(selectedImage.id);
        try {
            const result: any = await invoke('set_wallpaper', { imageUrl: url });

            if (result.success) {
                setSelectedImage(null);
            } else {
                alert('Failed: ' + result.error);
            }
        } catch (error) {
            console.error('Set wallpaper failed:', error);
            alert('Error: ' + error);
        } finally {
            setSettingWallpaper(null);
        }
    };

    return (
        <div style={{ padding: '40px', scrollBehavior: 'smooth' }}>
            {showWelcome && (
                <WelcomeModal
                    onClose={() => setShowWelcome(false)}
                    onSelectType={handleWelcomeChoice}
                />
            )}

            <motion.div
                initial={{ y: -20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ duration: 0.4 }}
                style={{ marginBottom: '36px' }}
            >
                {onBack && (
                    <motion.button
                        whileHover={{ x: -4 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={onBack}
                        style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '8px',
                            padding: '10px 20px',
                            background: 'var(--bg-secondary)',
                            border: '1px solid var(--border-color)',
                            borderRadius: '12px',
                            color: 'var(--text-secondary)',
                            cursor: 'pointer',
                            fontSize: '14px',
                            fontWeight: 600,
                            marginBottom: '24px',
                        }}
                    >
                        <ArrowLeft size={18} />
                        Back to Home
                    </motion.button>
                )}

                <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '28px' }}>
                    <h1 style={{ fontSize: '36px', fontWeight: 800, letterSpacing: '-0.02em', margin: 0 }}>
                        {currentType === 'live' ? 'Live 4k Wallpapers' : currentType === 'static' ? 'Static 4k Wallpapers' : 'All Wallpapers'}
                    </h1>
                    <motion.button
                        whileHover={{ scale: 1.2 }}
                        whileTap={{ scale: 0.9 }}
                        onClick={() => setShowWelcome(true)}
                        style={{
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            padding: 0,
                        }}
                        title="Change wallpaper type"
                    >
                        <ArrowLeftRight size={24} style={{ color: 'var(--text-secondary)' }} />
                    </motion.button>
                </div>

                <div style={{ display: 'flex', gap: '16px', maxWidth: '700px' }}>
                    <div style={{ position: 'relative', flex: 1 }}>
                        <Search
                            style={{
                                position: 'absolute',
                                left: '18px',
                                top: '50%',
                                transform: 'translateY(-50%)',
                                color: 'var(--text-tertiary)',
                                pointerEvents: 'none',
                            }}
                            size={20}
                        />
                        <input
                            type="text"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                            placeholder="Search wallpapers..."
                            style={{
                                width: '100%',
                                padding: '16px 20px 16px 52px',
                                background: 'var(--bg-secondary)',
                                border: '1px solid var(--border-color)',
                                borderRadius: '14px',
                                color: 'var(--text-primary)',
                                fontSize: '15px',
                                outline: 'none',
                                transition: 'border-color 0.2s',
                            }}
                        />
                    </div>
                    <motion.button
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={handleSearch}
                        style={{
                            padding: '16px 32px',
                            background: 'linear-gradient(135deg, var(--accent), var(--accent-hover))',
                            border: 'none',
                            borderRadius: '14px',
                            color: 'white',
                            fontSize: '15px',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        Search
                    </motion.button>
                </div>
            </motion.div>

            {loading && wallpapers.length === 0 ? (
                <div style={{ padding: '80px 0' }}>
                    <LoadingSpinner text="Loading wallpapers..." />
                </div>
            ) : wallpapers.length === 0 ? (
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    style={{
                        textAlign: 'center',
                        padding: '80px 20px',
                        color: 'var(--text-secondary)',
                    }}
                >
                    <h3 style={{ fontSize: '20px', fontWeight: 600, marginBottom: '8px' }}>No wallpapers found</h3>
                    <p style={{ fontSize: '14px' }}>Try a different search query</p>
                </motion.div>
            ) : (
                <>
                    <div
                        style={{
                            columnCount: 'auto',
                            columnWidth: '320px',
                            columnGap: '16px',
                            marginBottom: '40px',
                        }}
                    >
                        {wallpapers.map((wallpaper, index) => (
                            <motion.div
                                key={wallpaper.id}
                                initial={{ opacity: 0, y: 20 }}
                                animate={{ opacity: 1, y: 0 }}
                                transition={{ delay: index * 0.02, duration: 0.3 }}
                                onClick={() => setSelectedImage(wallpaper)}
                                style={{
                                    breakInside: 'avoid',
                                    marginBottom: '16px',
                                    cursor: 'pointer',
                                    borderRadius: '12px',
                                    overflow: 'hidden',
                                    position: 'relative',
                                }}
                            >
                                <motion.img
                                    whileHover={{ scale: 1.02 }}
                                    transition={{ duration: 0.2 }}
                                    src={wallpaper.thumbnailUrl}
                                    alt={wallpaper.title}
                                    style={{
                                        width: '100%',
                                        display: 'block',
                                        borderRadius: '12px',
                                        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.2)',
                                    }}
                                    loading="lazy"
                                />
                                {/* LIVE badge for video wallpapers */}
                                {wallpaper.type === 'video' && (
                                    <div
                                        style={{
                                            position: 'absolute',
                                            top: '12px',
                                            right: '12px',
                                            fontSize: '13px',
                                            fontWeight: 700,
                                            color: 'white',
                                            textShadow: '0 2px 8px rgba(0, 0, 0, 0.8)',
                                        }}
                                    >
                                        Live
                                    </div>
                                )}
                            </motion.div>
                        ))}
                    </div>

                    {loadingMore && (
                        <div style={{ textAlign: 'center', padding: '32px 0' }}>
                            <LoadingSpinner text="Loading more..." />
                        </div>
                    )}

                    {!hasMore && wallpapers.length > 0 && (
                        <div
                            style={{
                                textAlign: 'center',
                                padding: '32px 0',
                                color: 'var(--text-tertiary)',
                                fontSize: '14px',
                            }}
                        >
                            That's all for now :3 Contribute on github @shelleyloosespatience for more sources!
                        </div>
                    )}
                </>
            )}

            {selectedImage && (
                <ImageModal
                    image={selectedImage}
                    onClose={() => setSelectedImage(null)}
                    onSetWallpaper={handleSetWallpaper}
                    isLoading={settingWallpaper === selectedImage.id}
                />
            )}
        </div>
    );
}
