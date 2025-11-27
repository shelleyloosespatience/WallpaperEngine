import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { Search, ArrowLeft } from 'lucide-react';
import WallpaperCard from '../components/WallpaperCard';
import ImageModal from '../components/ImageModal';
import { LoadingSpinner } from '../components/LoadingState';
import { WallpaperItem } from '../types/wallpaper';

interface StorePageProps {
    selectedSource: string;
    filterType?: 'all' | 'live' | 'static';
    onBack?: () => void;
}

export default function StorePage({ selectedSource, filterType = 'all', onBack }: StorePageProps) {
    const [wallpapers, setWallpapers] = React.useState<WallpaperItem[]>([]);
    const [loading, setLoading] = React.useState(false);
    const [searchQuery, setSearchQuery] = React.useState('anime');
    const [hasMore, setHasMore] = React.useState(true);
    const [loadingMore, setLoadingMore] = React.useState(false);
    const [selectedImage, setSelectedImage] = React.useState<WallpaperItem | null>(null);
    const [settingWallpaper, setSettingWallpaper] = React.useState<string | null>(null);
    const pageRef = React.useRef(1);

    const getSourceName = () => {
        const names: Record<string, string> = {
            all: 'Static Wallpapers',
            wallhaven: 'WallHavenHD',
            wallpapers: 'WallpapersHD',
            wallpaperflare: 'Wallpaper4K',
            motionbgs: 'LiveWallpapers',
        };
        return filterType === 'live' ? 'Live Wallpapers' : names[selectedSource] || selectedSource;
    };

    const searchWallpapers = React.useCallback(
        async (pageNum: number = 1, append: boolean = false) => {
            if (append) {
                setLoadingMore(true);
            } else {
                setLoading(true);
                setWallpapers([]);
            }

            try {
                let sourcesToUse = selectedSource === 'all' ? undefined : [selectedSource];

                if (filterType === 'live') {
                    sourcesToUse = ['motionbgs'];
                }

                const result: any = await invoke('search_wallpapers', {
                    query: searchQuery || 'anime',
                    sources: sourcesToUse,
                    limitPerSource: 24,
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

                    let filtered = normalized;
                    if (filterType === 'live') {
                        filtered = normalized.filter((item: WallpaperItem) => item.type === 'video');
                    } else if (filterType === 'static') {
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
                console.error('sadly the search failed:', error);
                setHasMore(false);
            } finally {
                setLoading(false);
                setLoadingMore(false);
            }
        },
        [searchQuery, selectedSource, filterType]
    );

    React.useEffect(() => {
        searchWallpapers(1, false);
    }, [selectedSource, filterType]); 

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

        window.addEventListener('scroll', handleScroll);
        return () => window.removeEventListener('scroll', handleScroll);
    }, [loadingMore, loading, hasMore, searchWallpapers]);

    const handleSearch = () => {
        pageRef.current = 1;
        setHasMore(true);
        searchWallpapers(1, false);
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
            console.error('set wallpaper failed:', error);
            alert('Error: ' + error);
        } finally {
            setSettingWallpaper(null);
        }
    };

    return (
        <div style={{ padding: '40px' }}>
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

                <h1 style={{ fontSize: '36px', fontWeight: 800, marginBottom: '28px', letterSpacing: '-0.02em' }}>
                    {getSourceName()}
                </h1>

                <div style={{ display: 'flex', gap: '16px', maxWidth: '700px' }}>
                    <div style={{ position: 'relative', flex: 1 }}>
                        <Search
                            style={{
                                position: 'absolute',
                                left: '18px',
                                top: '50%',
                                transform: 'translateY(-50%)',
                                color: 'var(--text-tertiary)',
                            }}
                            size={20}
                        />
                        <input
                            type="text"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
                            placeholder="Search wallpapers..."
                            style={{
                                width: '100%',
                                padding: '16px 20px 16px 52px',
                                background: 'var(--bg-secondary)',
                                border: '1px solid var(--border-color)',
                                borderRadius: '16px',
                                color: 'var(--text-primary)',
                                fontSize: '15px',
                                outline: 'none',
                                transition: 'border 0.2s',
                            }}
                            onFocus={(e) => (e.currentTarget.style.borderColor = 'var(--accent)')}
                            onBlur={(e) => (e.currentTarget.style.borderColor = 'var(--border-color)')}
                        />
                    </div>

                    <motion.button
                        whileHover={{ scale: 1.05 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={handleSearch}
                        disabled={loading}
                        style={{
                            padding: '16px 32px',
                            background: loading ? 'var(--bg-tertiary)' : 'linear-gradient(135deg, var(--accent), #1a86d8)',
                            color: 'white',
                            border: 'none',
                            borderRadius: '16px',
                            cursor: loading ? 'not-allowed' : 'pointer',
                            fontWeight: 700,
                            fontSize: '15px',
                            boxShadow: loading ? 'none' : '0 4px 16px rgba(0, 120, 212, 0.3)',
                        }}
                    >
                        {loading ? 'Searching...' : 'Search'}
                    </motion.button>
                </div>
            </motion.div>

            {loading && wallpapers.length === 0 ? (
                <div style={{ padding: '80px 0' }}>
                    <LoadingSpinner text="Yaweee!! Discovering wallpapers..." />
                </div>
            ) : wallpapers.length === 0 ? (
                <motion.div
                    initial={{ scale: 0.95, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    style={{
                        textAlign: 'center',
                        padding: '100px 40px',
                        background: 'var(--bg-secondary)',
                        borderRadius: '24px',
                        border: '1px solid var(--border-color)',
                    }}
                >
                    <div style={{ fontSize: '80px', marginBottom: '20px', opacity: 0.2 }}>🔍</div>
                    <h2 style={{ fontSize: '24px', fontWeight: 700, marginBottom: '12px' }}>No wallpapers found</h2>
                    <p style={{ color: 'var(--text-secondary)', fontSize: '16px' }}>
                        Try a different search term
                    </p>
                </motion.div>
            ) : (
                <>
                    <div
                        style={{
                            columns: '4 300px',
                            columnGap: '24px',
                        }}
                    >
                        {wallpapers.map((wallpaper) => (
                            <WallpaperCard
                                key={wallpaper.id}
                                id={wallpaper.id}
                                thumbnail={wallpaper.thumbnailUrl || wallpaper.imageUrl}
                                type={wallpaper.type}
                                source={wallpaper.source}
                                onClick={() => setSelectedImage(wallpaper)}
                                isVisible={true}
                            />
                        ))}
                    </div>

                    {loadingMore && (
                        <div style={{ padding: '60px 0', textAlign: 'center' }}>
                            <LoadingSpinner text="Loading more..." />
                        </div>
                    )}

                    {!hasMore && wallpapers.length > 0 && (
                        <div
                            style={{
                                textAlign: 'center',
                                padding: '60px 0',
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
