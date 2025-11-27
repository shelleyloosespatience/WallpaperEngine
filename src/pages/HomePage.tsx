import React from 'react';
import { motion } from 'framer-motion';
import { Upload, Sparkles, Zap } from 'lucide-react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import StoreCard from '../components/StoreCard';

interface HomePageProps {
    onNavigateToSource: (source: string) => void;
    onNavigateToLive: () => void;
}

export default function HomePage({ onNavigateToSource, onNavigateToLive }: HomePageProps) {
    const [uploading, setUploading] = React.useState(false);

    const handleUpload = async () => {
        try {
            setUploading(true);
            const selected = await openDialog({
                multiple: false,
                filters: [
                    {
                        name: 'Media',
                        extensions: ['mp4', 'mkv', 'jpg', 'jpeg', 'png', 'gif', 'webm'],
                    },
                ],
            });

            if (selected && typeof selected === 'string') {
                const result: any = await invoke('upload_user_wallpaper', {
                    sourcePath: selected,
                });

                if (result.success) {
                    alert('✓ Wallpaper uploaded successfully!');
                } else {
                    alert('Failed to upload: ' + result.error);
                }
            }
        } catch (error) {
            console.error('Upload failed:', error);
            alert('Upload failed: ' + error);
        } finally {
            setUploading(false);
        }
    };

    return (
        <div style={{ padding: '48px 40px' }}>
            <motion.div
                initial={{ y: -30, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ duration: 0.6, ease: 'easeOut' }}
                style={{ marginBottom: '48px' }}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '12px' }}>
                    <Sparkles size={32} style={{ color: 'var(--accent)' }} />
                    <h1 style={{ fontSize: '42px', fontWeight: 800, letterSpacing: '-0.02em' }}>
                        Welcome to Colorwall! 
                    </h1>
                </div>
                <p style={{ color: 'var(--text-secondary)', fontSize: '18px', lineHeight: 1.6 }}>
                    An open source wallpaper engine, made for performance and easy usage!
                </p>
            </motion.div>

            {/* Upload Section */}
            <motion.div
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.2, duration: 0.6 }}
                style={{
                    background: 'linear-gradient(135deg, rgba(0, 120, 212, 0.1), rgba(99, 102, 241, 0.1))',
                    borderRadius: '24px',
                    padding: '40px',
                    marginBottom: '48px',
                    border: '1px solid rgba(0, 120, 212, 0.2)',
                    backdropFilter: 'blur(20px)',
                }}
            >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ flex: 1 }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '12px' }}>
                            <Upload size={28} style={{ color: 'var(--accent)' }} />
                            <h2 style={{ fontSize: '24px', fontWeight: 700 }}>Upload Your Own</h2>
                        </div>
                        <p style={{ color: 'var(--text-secondary)', fontSize: '15px', maxWidth: '500px' }}>
                            Personalize your desktop with your favorite images and lively wallpapers!
                        </p>
                    </div>
                    <motion.button
                        whileHover={{ scale: 1.05, boxShadow: '0 12px 40px rgba(0, 120, 212, 0.4)' }}
                        whileTap={{ scale: 0.95 }}
                        onClick={handleUpload}
                        disabled={uploading}
                        style={{
                            padding: '16px 32px',
                            background: uploading
                                ? 'var(--bg-tertiary)'
                                : 'linear-gradient(135deg, var(--accent), #1a86d8)',
                            color: 'white',
                            border: 'none',
                            borderRadius: '16px',
                            cursor: uploading ? 'not-allowed' : 'pointer',
                            fontWeight: 700,
                            fontSize: '16px',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '10px',
                        }}
                    >
                        {uploading ? (
                            <>
                                <div
                                    style={{
                                        width: '20px',
                                        height: '20px',
                                        border: '2px solid rgba(255,255,255,0.3)',
                                        borderTop: '2px solid white',
                                        borderRadius: '50%',
                                        animation: 'spin 0.8s linear infinite',
                                    }}
                                />
                                Uploading...
                            </>
                        ) : (
                            <>
                                <Upload size={20} />
                                Choose File
                            </>
                        )}
                    </motion.button>
                </div>
            </motion.div>

            {/* Browse Section */}
            <motion.div
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.3, duration: 0.6 }}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '28px' }}>
                    <Zap size={24} style={{ color: 'var(--accent)' }} />
                    <h2 style={{ fontSize: '28px', fontWeight: 700 }}>Browse Store</h2>
                </div>

                <div
                    style={{
                        display: 'grid',
                        gridTemplateColumns: 'repeat(auto-fit, minmax(380px, 1fr))',
                        gap: '24px',
                    }}
                >
                    <StoreCard
                        title="Static Wallpapers"
                        description="High-quality images from WallHaven, MoeWalls, and more"
                        imagePath="/assets/all-sources.png"
                        type="static"
                        onClick={() => onNavigateToSource('all')}
                    />
                    <StoreCard
                        title="Live Wallpapers"
                        description="Animated wallpapers that bring your desktop to life"
                        imagePath="/assets/live-wallpapers.png"
                        type="live"
                        onClick={onNavigateToLive}
                    />
                </div>
            </motion.div>
        </div>
    );
}
