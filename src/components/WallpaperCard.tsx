import { useEffect, useRef, useState } from 'react';
import { motion } from 'framer-motion';
import { Image, Play, ZoomIn } from 'lucide-react';

interface WallpaperCardProps {
    id: string;
    thumbnail?: string;
    type: 'image' | 'video';
    source?: string;
    isActive?: boolean;
    isVisible?: boolean;
    onClick?: () => void;
    onSet?: () => void;
    onDelete?: () => void;
}

export default function WallpaperCard({
    thumbnail,
    type,
    source = 'unknown',
    isActive,
    isVisible = true,
    onClick,
    onSet,
    onDelete,
}: WallpaperCardProps) {
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
        if (shouldLoad && imgRef.current && !imgRef.current.src && thumbnail) {
            imgRef.current.src = thumbnail;
        }
    }, [shouldLoad, thumbnail]);

    return (
        <motion.div
            ref={cardRef}
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            whileHover={{ y: -8, scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={onClick}
            style={{
                position: 'relative',
                borderRadius: '20px',
                overflow: 'hidden',
                cursor: onClick ? 'pointer' : 'default',
                border: isActive ? '3px solid var(--accent)' : 'none',
                breakInside: 'avoid',
                marginBottom: '20px',
                boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
            }}
            className="group"
        >
            <div
                style={{
                    position: 'relative',
                    overflow: 'hidden',
                    background: '#0a0a0a',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                }}
            >
                {!imgLoaded && !imgError && shouldLoad && (
                    <div
                        style={{
                            position: 'absolute',
                            inset: 0,
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            minHeight: '200px',
                        }}
                    >
                        <div
                            style={{
                                width: '32px',
                                height: '32px',
                                border: '3px solid rgba(0, 120, 212, 0.2)',
                                borderTop: '3px solid var(--accent)',
                                borderRadius: '50%',
                                animation: 'spin 0.8s linear infinite',
                            }}
                        />
                    </div>
                )}

                {shouldLoad ? (
                    <>
                        {type === 'video' ? (
                            <video
                                ref={imgRef as any}
                                src={thumbnail}
                                style={{
                                    width: '100%',
                                    height: 'auto',
                                    display: 'block',
                                    transition: 'all 0.6s cubic-bezier(0.4, 0, 0.2, 1)',
                                    opacity: imgLoaded ? 1 : 0,
                                }}
                                className="group-hover:scale-110"
                                muted
                                loop
                                playsInline
                                onLoadedData={() => setImgLoaded(true)}
                                onError={() => setImgError(true)}
                                onMouseEnter={(e) => {
                                    const video = e.currentTarget;
                                    video.play().catch(() => { });
                                }}
                                onMouseLeave={(e) => {
                                    const video = e.currentTarget;
                                    video.pause();
                                    video.currentTime = 0;
                                }}
                                onContextMenu={(e) => {
                                    e.preventDefault();
                                    return false;
                                }}
                            />
                        ) : (
                            <img
                                ref={imgRef}
                                alt="Wallpaper"
                                style={{
                                    width: '100%',
                                    height: 'auto',
                                    display: 'block',
                                    transition: 'all 0.6s cubic-bezier(0.4, 0, 0.2, 1)',
                                    opacity: imgLoaded ? 1 : 0,
                                }}
                                className="group-hover:scale-110"
                                loading="lazy"
                                onLoad={() => setImgLoaded(true)}
                                onError={() => setImgError(true)}
                                onContextMenu={(e) => {
                                    e.preventDefault();
                                    return false;
                                }}
                                draggable={false}
                            />
                        )}
                    </>
                ) : (
                    <div
                        style={{
                            width: '100%',
                            minHeight: '200px',
                            background: 'linear-gradient(135deg, #0a0a0a, #1a1a1a)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                        }}
                    >
                        <div
                            style={{
                                width: '32px',
                                height: '32px',
                                border: '3px solid rgba(128, 128, 128, 0.2)',
                                borderTop: '3px solid rgba(128, 128, 128, 0.4)',
                                borderRadius: '50%',
                                animation: 'spin 0.8s linear infinite',
                            }}
                        />
                    </div>
                )}

                {imgError && (
                    <div
                        style={{
                            minHeight: '200px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            color: 'var(--text-tertiary)',
                        }}
                    >
                        <div style={{ textAlign: 'center' }}>
                            <Image style={{ width: '32px', height: '32px', margin: '0 auto 12px', opacity: 0.3 }} />
                            <span style={{ fontSize: '13px', opacity: 0.5 }}>Failed to load</span>
                        </div>
                    </div>
                )}

                <div
                    style={{
                        position: 'absolute',
                        inset: 0,
                        background: 'linear-gradient(to top, rgba(0, 0, 0, 0.9) 0%, rgba(0, 0, 0, 0.4) 30%, transparent 70%)',
                        opacity: 0,
                        transition: 'opacity 0.4s cubic-bezier(0.4, 0, 0.2, 1)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                    }}
                    className="group-hover:opacity-100"
                >
                    {type === 'video' && source === 'motionbgs' ? (
                        <motion.div
                            initial={{ y: 10, opacity: 0 }}
                            whileHover={{ y: 0, opacity: 1 }}
                            style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '10px',
                                background: 'rgba(16, 185, 129, 0.95)',
                                padding: '14px 24px',
                                borderRadius: '50px',
                                fontSize: '15px',
                                fontWeight: 700,
                                backdropFilter: 'blur(20px)',
                                boxShadow: '0 8px 32px rgba(16, 185, 129, 0.4)',
                            }}
                        >
                            <Play size={18} strokeWidth={3} />
                            Live Wallpaper
                        </motion.div>
                    ) : type === 'video' ? (
                        <motion.div
                            initial={{ y: 10, opacity: 0 }}
                            whileHover={{ y: 0, opacity: 1 }}
                            style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '10px',
                                background: 'rgba(168, 85, 247, 0.95)',
                                padding: '14px 24px',
                                borderRadius: '50px',
                                fontSize: '15px',
                                fontWeight: 700,
                                backdropFilter: 'blur(20px)',
                                boxShadow: '0 8px 32px rgba(168, 85, 247, 0.4)',
                            }}
                        >
                            <Play size={18} strokeWidth={3} />
                            Animated
                        </motion.div>
                    ) : (
                        <motion.div
                            initial={{ y: 10, opacity: 0 }}
                            whileHover={{ y: 0, opacity: 1 }}
                            style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '10px',
                                background: 'rgba(0, 120, 212, 0.95)',
                                padding: '14px 24px',
                                borderRadius: '50px',
                                fontSize: '15px',
                                fontWeight: 700,
                                backdropFilter: 'blur(20px)',
                                boxShadow: '0 8px 32px rgba(0, 120, 212, 0.4)',
                            }}
                        >
                            <ZoomIn size={18} strokeWidth={3} />
                            View
                        </motion.div>
                    )}
                </div>

                {isActive && (
                    <motion.div
                        initial={{ scale: 0 }}
                        animate={{ scale: 1 }}
                        style={{
                            position: 'absolute',
                            top: '16px',
                            right: '16px',
                            padding: '8px 16px',
                            background: 'rgba(0, 120, 212, 0.95)',
                            borderRadius: '50px',
                            fontSize: '12px',
                            fontWeight: 700,
                            color: 'white',
                            backdropFilter: 'blur(20px)',
                            boxShadow: '0 4px 16px rgba(0, 120, 212, 0.5)',
                        }}
                    >
                        ✓ Active
                    </motion.div>
                )}

                {(onSet || onDelete) && (
                    <div
                        style={{
                            position: 'absolute',
                            bottom: '16px',
                            left: '16px',
                            right: '16px',
                            display: 'flex',
                            gap: '12px',
                            opacity: 0,
                            transform: 'translateY(10px)',
                            transition: 'all 0.3s',
                        }}
                        className="group-hover:opacity-100"
                        onMouseEnter={(e) => {
                            e.currentTarget.style.transform = 'translateY(0)';
                        }}
                        onMouseLeave={(e) => {
                            e.currentTarget.style.transform = 'translateY(10px)';
                        }}
                    >
                        {onSet && (
                            <motion.button
                                whileHover={{ scale: 1.05 }}
                                whileTap={{ scale: 0.95 }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onSet();
                                }}
                                style={{
                                    flex: 1,
                                    padding: '12px',
                                    fontSize: '14px',
                                    background: 'rgba(0, 120, 212, 0.95)',
                                    color: 'white',
                                    borderRadius: '12px',
                                    border: 'none',
                                    cursor: 'pointer',
                                    fontWeight: 700,
                                    backdropFilter: 'blur(20px)',
                                    boxShadow: '0 4px 16px rgba(0, 120, 212, 0.4)',
                                }}
                            >
                                Set
                            </motion.button>
                        )}
                        {onDelete && (
                            <motion.button
                                whileHover={{ scale: 1.05, backgroundColor: 'rgba(220, 38, 38, 0.95)' }}
                                whileTap={{ scale: 0.95 }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onDelete();
                                }}
                                style={{
                                    padding: '12px 16px',
                                    fontSize: '14px',
                                    background: 'rgba(62, 62, 66, 0.95)',
                                    color: 'white',
                                    borderRadius: '12px',
                                    border: 'none',
                                    cursor: 'pointer',
                                    backdropFilter: 'blur(20px)',
                                    transition: 'background 0.2s',
                                }}
                            >
                                🗑️
                            </motion.button>
                        )}
                    </div>
                )}
            </div>
        </motion.div>
    );
}
