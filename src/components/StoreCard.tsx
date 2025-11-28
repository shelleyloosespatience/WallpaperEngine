import { motion } from 'framer-motion';
import { ArrowRight, Play, Image as ImageIcon } from 'lucide-react';

interface StoreCardProps {
    title: string;
    description: string;
    imagePath: string;
    type: 'live' | 'static';
    onClick: () => void;
}

export default function StoreCard({ title, description, imagePath, type, onClick }: StoreCardProps) {
    const isVideo = imagePath.match(/\.(mp4|webm|mkv)$/i);

    return (
        <motion.div
            whileHover={{ scale: 1.02, y: -4 }}
            whileTap={{ scale: 0.98 }}
            onClick={onClick}
            style={{
                background: 'var(--bg-secondary)',
                borderRadius: '16px',
                overflow: 'hidden',
                cursor: 'pointer',
                border: '1px solid var(--border-color)',
                transition: 'all 0.3s',
            }}
        >
            <div
                style={{
                    width: '100%',
                    height: '280px',
                    position: 'relative',
                    overflow: 'hidden',
                    background: '#000',
                }}
            >
                {/* Render video or image based on file type */}
                {isVideo ? (
                    <video
                        src={imagePath}
                        autoPlay
                        loop
                        muted
                        playsInline
                        style={{
                            width: '100%',
                            height: '100%',
                            objectFit: 'cover',
                        }}
                    />
                ) : (
                    <img
                        src={imagePath}
                        alt={title}
                        style={{
                            width: '100%',
                            height: '100%',
                            objectFit: 'cover',
                        }}
                    />
                )}
                
                {/* Overlay gradient for better text readability */}
                <div
                    style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        right: 0,
                        bottom: 0,
                        background: 'linear-gradient(to bottom, rgba(0,0,0,0.3), rgba(0,0,0,0.1))',
                        pointerEvents: 'none',
                    }}
                />
                
                <div
                    style={{
                        position: 'absolute',
                        top: '16px',
                        right: '16px',
                        background: 'rgba(0, 0, 0, 0.6)',
                        backdropFilter: 'blur(10px)',
                        padding: '6px 12px',
                        borderRadius: '20px',
                        fontSize: '12px',
                        fontWeight: 600,
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        color: 'white',
                    }}
                >
                    {type === 'live' ? <Play size={14} /> : <ImageIcon size={14} />}
                    {type === 'live' ? 'Live' : 'Static'}
                </div>
            </div>
            <div style={{ padding: '24px' }}>
                <h3 style={{ fontSize: '20px', fontWeight: 600, marginBottom: '8px' }}>{title}</h3>
                <p style={{ color: 'var(--text-secondary)', fontSize: '14px', marginBottom: '16px' }}>
                    {description}
                </p>
                <motion.div
                    whileHover={{ gap: '12px' }}
                    style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        color: 'var(--accent)',
                        fontSize: '14px',
                        fontWeight: 600,
                        transition: 'gap 0.2s',
                    }}
                >
                    Browse {type === 'live' ? 'Live' : 'Static'} Wallpapers
                    <ArrowRight size={16} />
                </motion.div>
            </div>
        </motion.div>
    );
}