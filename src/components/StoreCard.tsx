import { motion } from 'framer-motion';
import { ArrowRight, Play, Image as ImageIcon } from 'lucide-react';

interface StoreCardProps {
    title: string;
    description: string;
    imagePath: string;
    type: 'live' | 'static';
    onClick: () => void;
}

export default function StoreCard({ title, description, type, onClick }: StoreCardProps) {
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
                    background: `linear-gradient(135deg, ${type === 'live' ? '#4f46e5, #7c3aed' : '#0ea5e9, #06b6d4'})`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    position: 'relative',
                    overflow: 'hidden',
                }}
            >
                <div style={{ fontSize: '80px', opacity: 0.2 }}>
                    {type === 'live' ? <Play size={80} /> : <ImageIcon size={80} />}
                </div>
                <div
                    style={{
                        position: 'absolute',
                        top: '16px',
                        right: '16px',
                        background: 'rgba(0, 0, 0, 0.4)',
                        backdropFilter: 'blur(10px)',
                        padding: '6px 12px',
                        borderRadius: '20px',
                        fontSize: '12px',
                        fontWeight: 600,
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
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
