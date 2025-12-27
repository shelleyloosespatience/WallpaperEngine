import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Download, Loader2, Play, X, ZoomIn, ZoomOut, CheckCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { WallpaperItem } from '../types/wallpaper';
// import { getSourceIcon } from './icons';
import { motion, AnimatePresence } from 'framer-motion';

interface ImageModalProps {
    image: WallpaperItem;
    onClose: () => void;
    onSetWallpaper: (url: string) => void;
    isLoading: boolean;
}

const ImageModal = ({ image, onClose, onSetWallpaper, isLoading }: ImageModalProps) => {
    const [zoom, setZoom] = useState(1);
    const [imgLoaded, setImgLoaded] = useState(false);
    const [displayUrl, setDisplayUrl] = useState<string>(image.thumbnailUrl || image.imageUrl);
    const [highResUrl, setHighResUrl] = useState<string | null>(null);
    const [url4k, setUrl4k] = useState<string | null>(null);
    const [isResolving, setIsResolving] = useState(false);
    const [isSettingVideo, setIsSettingVideo] = useState(false);
    const [progressMessage, setProgressMessage] = useState<string>('');
    const hasResolvedRef = useRef(false);
    const abortControllerRef = useRef<AbortController | null>(null);

    useEffect(() => {
        hasResolvedRef.current = false;
        setIsResolving(false);

        if (image.source === 'wallpaperflare' && image.detailUrl) {
            setIsResolving(true);
            (async () => {
                try {
                    const result: any = await invoke('resolve_wallpaperflare_highres', { detailUrl: image.detailUrl });
                    if (result?.success && result?.url && !hasResolvedRef.current) {
                        hasResolvedRef.current = true;
                        setHighResUrl(result.url);
                        setDisplayUrl(result.url);
                        setImgLoaded(false);
                        setIsResolving(false);
                    }
                } catch (e) {
                    console.error('[ERROR] Failed to resolve high-res:', e);
                    setIsResolving(false);
                }
            })();
        } else if (image.source === 'motionbgs' && image.detailUrl) {
            setIsResolving(true);
            (async () => {
                try {
                    const result: any = await invoke('resolve_motionbgs_video', { detailUrl: image.detailUrl });
                    if (result?.success && result?.url && !hasResolvedRef.current) {
                        hasResolvedRef.current = true;
                        setHighResUrl(result.url);
                        setDisplayUrl(result.url);
                        setUrl4k(result.url4k || null);
                        setImgLoaded(false);
                        setIsResolving(false);
                    }
                } catch (e) {
                    console.error('[ERROR] Failed to resolve video:', e);
                    setIsResolving(false);
                }
            })();
        } else {
            setHighResUrl(image.imageUrl);
        }
    }, [image.id, image.detailUrl, image.source, image.thumbnailUrl, image.imageUrl]);

    useEffect(() => {
        const handleEsc = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };
        window.addEventListener('keydown', handleEsc);
        return () => window.removeEventListener('keydown', handleEsc);
    }, [onClose]);

    const urlForWallpaper = highResUrl || displayUrl;

    const handleContextMenu = useCallback((e: React.MouseEvent) => {
        e.preventDefault();
        return false;
    }, []);

    const isMp4Video = useCallback(
        (url?: string) => !!url && image.type === 'video' && /\.mp4($|\?)/i.test(url),
        [image.type]
    );

    return (
        <AnimatePresence>
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.2 }}
                className="fixed inset-0 z-[9999] bg-black/98 backdrop-blur-xl flex pt-8"
                onClick={onClose}
                onContextMenu={handleContextMenu}
            >
                <div className="flex-1 flex items-center justify-center p-8 pt-16 relative" onClick={(e) => e.stopPropagation()}>
                    <motion.div
                        initial={{ y: 20, opacity: 0 }}
                        animate={{ y: 0, opacity: 1 }}
                        transition={{ delay: 0.1 }}
                        className="absolute top-14 left-6 z-10 flex gap-2"
                    >
                        <button
                            onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                                e.stopPropagation();
                                setZoom(Math.min(zoom + 0.25, 3));
                            }}
                            className="p-2.5 bg-gray-900/95 hover:bg-gray-800 rounded-lg transition-all cursor-pointer border border-gray-800/50 shadow-lg backdrop-blur-sm"
                        >
                            <ZoomIn className="w-4 h-4 text-gray-300" />
                        </button>
                        <button
                            onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                                e.stopPropagation();
                                setZoom(Math.max(zoom - 0.25, 0.5));
                            }}
                            className="p-2.5 bg-gray-900/95 hover:bg-gray-800 rounded-lg transition-all cursor-pointer border border-gray-800/50 shadow-lg backdrop-blur-sm"
                        >
                            <ZoomOut className="w-4 h-4 text-gray-300" />
                        </button>
                        <button
                            onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                                e.stopPropagation();
                                setZoom(1);
                            }}
                            className="px-3 py-2.5 bg-gray-900/95 hover:bg-gray-800 rounded-lg transition-all text-xs text-gray-300 cursor-pointer border border-gray-800/50 shadow-lg backdrop-blur-sm font-medium"
                        >
                            Reset
                        </button>
                    </motion.div>

                    {isResolving && (
                        <motion.div
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent pointer-events-none z-20"
                        >
                            <motion.div
                                animate={{ scale: [1, 1.05, 1] }}
                                transition={{ repeat: Infinity, duration: 2 }}
                                className="absolute bottom-8 left-1/2 -translate-x-1/2 flex items-center gap-3 bg-gradient-to-r from-blue-600/90 via-indigo-600/90 to-purple-600/90 backdrop-blur-md px-5 py-3 rounded-full border border-blue-400/30 shadow-2xl shadow-blue-500/50"
                            >
                                <div className="relative">
                                    <Loader2 className="w-5 h-5 animate-spin text-white" />
                                    <div className="absolute inset-0 blur-md bg-white/30 rounded-full animate-spin" />
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-sm text-white font-bold tracking-wide">Fetching HD Quality</span>
                                    <span className="text-xs text-blue-100">Please wait...</span>
                                </div>
                            </motion.div>
                        </motion.div>
                    )}

                    {isMp4Video(highResUrl || displayUrl) ? (
                        <video
                            key={highResUrl || displayUrl}
                            src={highResUrl || displayUrl}
                            controls
                            autoPlay
                            loop
                            className={`max-w-full max-h-full object-contain transition-all duration-500 rounded-lg ${imgLoaded ? 'opacity-100' : 'opacity-0'
                                }`}
                            style={{ transform: `scale(${zoom})` }}
                            onLoadedData={() => setImgLoaded(true)}
                            onError={() => setImgLoaded(true)}
                            onContextMenu={handleContextMenu}
                            draggable={false}
                        />
                    ) : (
                        <motion.img
                            key={displayUrl}
                            initial={{ opacity: 0, scale: 0.95 }}
                            animate={{ opacity: imgLoaded ? 1 : 0, scale: imgLoaded ? 1 : 0.95 }}
                            src={displayUrl}
                            alt={image.title || 'Wallpaper'}
                            className="max-w-full max-h-full object-contain rounded-lg"
                            style={{ transform: `scale(${zoom})` }}
                            onLoad={() => setImgLoaded(true)}
                            onError={() => setImgLoaded(true)}
                            onContextMenu={handleContextMenu}
                            draggable={false}
                        />
                    )}
                </div>

                <motion.div
                    initial={{ x: 100, opacity: 0 }}
                    animate={{ x: 0, opacity: 1 }}
                    exit={{ x: 100, opacity: 0 }}
                    transition={{ type: 'spring', damping: 25 }}
                    className="w-96 bg-black/95 backdrop-blur-2xl border-l border-gray-900 flex flex-col shadow-2xl pt-8"
                    onClick={(e) => e.stopPropagation()}
                    onContextMenu={handleContextMenu}
                >
                    <div className="px-4 py-3 border-b border-gray-900/50">
                        <button
                            onClick={onClose}
                            className="w-full flex items-center justify-center gap-2 p-2.5 bg-gray-900/80 hover:bg-gray-800 rounded-lg transition-all cursor-pointer border border-gray-800/50 group"
                        >
                            <X className="w-4 h-4 text-gray-500 group-hover:text-gray-300 transition-colors" />
                            <span className="text-xs font-medium text-gray-500 group-hover:text-gray-300 transition-colors">Close</span>
                        </button>
                    </div>

                    <div className="flex-1 overflow-y-auto p-4 space-y-4">
                        <div>
                            <h3 className="text-base font-bold text-gray-200 mb-2">{image.title || 'Untitled'}</h3>
                            <div className="inline-flex items-center gap-2 bg-blue-500/10 px-2.5 py-1.5 rounded-lg border border-blue-500/20">
                                {/* {getSourceIcon(image.source)} */}
                                <span className="text-xs font-bold uppercase tracking-wider text-blue-400">{image.source}</span>
                            </div>
                        </div>

                        {image.width && image.height && (
                            <div className="bg-gray-900/50 rounded-lg p-3 border border-gray-800/50">
                                <div className="text-xs text-gray-500 font-semibold uppercase tracking-wider mb-1.5">Dimensions</div>
                                <div className="text-xl font-bold text-gray-200 font-mono">
                                    {image.width} × {image.height}
                                </div>
                                <div className="text-xs text-gray-600 mt-1">{(image.width / image.height).toFixed(2)} aspect ratio</div>
                            </div>
                        )}

                        {image.tags && image.tags.length > 0 && (
                            <div className="bg-gray-900/50 rounded-lg p-3 border border-gray-800/50">
                                <div className="text-xs text-gray-500 font-semibold uppercase tracking-wider mb-2">Tags</div>
                                <div className="flex flex-wrap gap-1.5">
                                    {image.tags.slice(0, 10).map((tag, idx) => (
                                        <span
                                            key={idx}
                                            className="px-2 py-1 bg-gray-800/50 text-gray-400 rounded-md text-xs font-medium border border-gray-700/50"
                                        >
                                            {tag}
                                        </span>
                                    ))}
                                </div>
                            </div>
                        )}

                        {image.type === 'video' && (
                            <div className="bg-emerald-500/10 rounded-lg p-3 border border-emerald-500/20">
                                <div className="flex items-center gap-2 text-emerald-400">
                                    <Play className="w-4 h-4" />
                                    <span className="text-sm font-semibold">Live Wallpaper</span>
                                </div>
                            </div>
                        )}
                    </div>

                    <div className="p-4 border-t border-gray-900/50 space-y-2">
                        <a
                            href={url4k || urlForWallpaper}
                            download
                            onClick={(e: React.MouseEvent<HTMLAnchorElement>) => e.stopPropagation()}
                            onContextMenu={handleContextMenu}
                            className="w-full flex items-center justify-center gap-2 bg-gray-900/80 hover:bg-gray-800 text-white px-4 py-3 rounded-lg transition-all font-medium shadow-lg cursor-pointer border border-gray-800/50 text-sm"
                        >
                            <Download className="w-4 h-4" />
                            {url4k ? 'Download 4K' : 'Download'}
                        </a>

                        {image.type === 'video' && image.source === 'motionbgs' ? (
                            <button
                                onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                                    e.stopPropagation();
                                    const videoUrlToUse = url4k || highResUrl;
                                    if (!videoUrlToUse) {
                                        alert('Video URL not resolved yet. Please wait...');
                                        return;
                                    }
   
                                    // cancel
                                    if (abortControllerRef.current) {
                                        abortControllerRef.current.abort();
                                        console.log('[ImageModal] Cancelled previous operation');
                                    }
                                    abortControllerRef.current = new AbortController();

                                    setIsSettingVideo(true);
                                    setProgressMessage('Initializing...');

                                    // yeawee progress updates (since backend doesnt stream progress)
                                    const progressSteps = [
                                        { delay: 0, message: 'Preparing video...' },
                                        { delay: 2500, message: 'Downloading video...' },
                                        { delay: 9000, message: 'Processing file...' },
                                        { delay: 12000, message: 'Setting wallpaper...' },
                                    ];

                                    let currentStep = 0;
                                    const progressInterval = setInterval(() => {
                                        if (currentStep < progressSteps.length && !abortControllerRef.current?.signal.aborted) {
                                            setProgressMessage(progressSteps[currentStep].message);
                                            currentStep++;
                                        }
                                    }, 1500);

                                    invoke('set_video_wallpaper', { videoUrl: videoUrlToUse })
                                        .then((result: any) => {
                                            clearInterval(progressInterval);
                                            if (!abortControllerRef.current?.signal.aborted) {
                                                setIsSettingVideo(false);
                                                setProgressMessage('');
                                                if (result.success) {
                                                    onClose();
                                                } else {
                                                    alert('Failed: ' + (result.error || 'Unknown error'));
                                                }
                                            }
                                        })
                                        .catch((err) => {
                                            clearInterval(progressInterval);
                                            if (!abortControllerRef.current?.signal.aborted) {
                                                setIsSettingVideo(false);
                                                setProgressMessage('');
                                                alert('Error: ' + err);
                                            }
                                        });
                                }}
                                disabled={isLoading || isSettingVideo || !highResUrl}
                                className="w-full flex items-center justify-center gap-2 bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 disabled:opacity-50 disabled:cursor-not-allowed text-white px-4 py-3 rounded-lg transition-all font-medium shadow-xl shadow-emerald-500/20 cursor-pointer text-sm"
                            >
                                {isSettingVideo ? (
                                    <>
                                        <Loader2 className="w-4 h-4 animate-spin" />
                                        <span className="text-xs">{progressMessage || 'Starting...'}</span>
                                    </>
                                ) : (
                                    <>
                                        <Play className="w-5 h-5" />
                                        {url4k ? 'Set 4K Live Wallpaper' : 'Set Live Wallpaper'}
                                    </>
                                )}
                            </button>
                        ) : image.type === 'video' ? (
                            <button
                                onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                                    e.stopPropagation();
                                    window.open(image.imageUrl, '_blank');
                                }}
                                className="w-full flex items-center justify-center gap-2 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 text-white px-4 py-3 rounded-lg transition-all font-medium shadow-xl shadow-purple-500/20 cursor-pointer text-sm"
                            >
                                <Play className="w-5 h-5" />
                                View Live Wallpaper
                            </button>
                        ) : (
                            <button
                                onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                                    e.stopPropagation();
                                    onSetWallpaper(urlForWallpaper);
                                }}
                                disabled={isLoading}
                                className="w-full flex items-center justify-center gap-2 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed text-white px-4 py-3 rounded-lg transition-all font-medium shadow-xl shadow-blue-500/20 cursor-pointer text-sm"
                            >
                                {isLoading ? (
                                    <>
                                        <Loader2 className="w-4 h-4 animate-spin" />
                                        Yaweeee! Setting...
                                    </>
                                ) : (
                                    <>
                                        <CheckCircle className="w-5 h-5" />
                                        Set as Wallpaper?
                                    </>
                                )}
                            </button>
                        )}
                    </div>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
};

export default ImageModal;