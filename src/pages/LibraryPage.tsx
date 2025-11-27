import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { motion } from 'framer-motion';
import { Upload } from 'lucide-react';
import WallpaperCard from '../components/WallpaperCard';
import { LoadingSpinner } from '../components/LoadingState';

interface UserWallpaper {
    id: string;
    name: string;
    path: string;
    media_type: string;
    thumbnail?: string;
    added_at: number;
}

interface VideoWallpaperState {
    isActive: boolean;
    videoPath?: string;
    videoUrl?: string;
}

export default function LibraryPage() {
    const [wallpapers, setWallpapers] = React.useState<UserWallpaper[]>([]);
    const [loading, setLoading] = React.useState(true);
    const [currentWallpaper, setCurrentWallpaper] = React.useState('');
    const [videoState, setVideoState] = React.useState<VideoWallpaperState>({ isActive: false });
    const [uploading, setUploading] = React.useState(false);

    const loadWallpapers = React.useCallback(async () => {
        try {
            const result: any = await invoke('list_user_wallpapers');
            if (result.success) {
                setWallpapers(result.wallpapers || []);
            }
        } catch (error) {
            console.error('Failed to load wallpapers:', error);
        }
    }, []);

    const loadCurrentState = React.useCallback(async () => {
        try {
            const [currentWp, videoSt]: any = await Promise.all([
                invoke('get_current_wallpaper'),
                invoke('get_video_wallpaper_status'),
            ]);

            if (currentWp?.success && currentWp?.message) {
                setCurrentWallpaper(currentWp.message);
            }

            if (videoSt) {
                setVideoState(videoSt);
            }
        } catch (error) {
            console.error('Failed to load current state:', error);
        }
    }, []);

    React.useEffect(() => {
        (async () => {
            setLoading(true);
            await Promise.all([loadWallpapers(), loadCurrentState()]);
            setLoading(false);
        })();
    }, [loadWallpapers, loadCurrentState]);

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
                    await loadWallpapers();
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

    const handleSetWallpaper = async (wallpaper: UserWallpaper) => {
        try {
            if (wallpaper.media_type === 'video') {
                const result: any = await invoke('set_video_wallpaper', {
                    videoUrl: `file://${wallpaper.path}`,
                });

                if (result.success) {
                    await loadCurrentState();
                } else {
                    alert('Failed: ' + result.error);
                }
            } else {
                const result: any = await invoke('set_wallpaper', {
                    imageUrl: `file://${wallpaper.path}`,
                });

                if (result.success) {
                    await loadCurrentState();
                } else {
                    alert('Failed: ' + result.error);
                }
            }
        } catch (error) {
            console.error('Set wallpaper failed:', error);
            alert('Error: ' + error);
        }
    };

    const handleDelete = async (wallpaper: UserWallpaper) => {
        if (!confirm(`Delete "${wallpaper.name}"?`)) return;

        try {
            const result: any = await invoke('delete_user_wallpaper', {
                wallpaperPath: wallpaper.path,
            });

            if (result.success) {
                await loadWallpapers();
            } else {
                alert('Failed to delete: ' + result.error);
            }
        } catch (error) {
            console.error('Delete failed:', error);
            alert('Error: ' + error);
        }
    };

    const isActive = (wallpaper: UserWallpaper) => {
        if (wallpaper.media_type === 'video') {
            return videoState.isActive && videoState.videoPath === wallpaper.path;
        } else {
            return currentWallpaper.includes(wallpaper.name) || currentWallpaper === wallpaper.path;
        }
    };

    if (loading) {
        return (
            <div style={{ padding: '80px 0' }}>
                <LoadingSpinner text="Loading your collection..." />
            </div>
        );
    }

    return (
        <div style={{ padding: '40px' }}>
            <motion.div
                initial={{ y: -20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ duration: 0.4 }}
                style={{ marginBottom: '36px' }}
            >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <h1 style={{ fontSize: '36px', fontWeight: 800, letterSpacing: '-0.02em' }}>
                        My Collection
                    </h1>
                    <motion.button
                        whileHover={{ scale: 1.05, boxShadow: '0 8px 32px rgba(0, 120, 212, 0.3)' }}
                        whileTap={{ scale: 0.95 }}
                        onClick={handleUpload}
                        disabled={uploading}
                        style={{
                            padding: '14px 28px',
                            background: uploading
                                ? 'var(--bg-tertiary)'
                                : 'linear-gradient(135deg, var(--accent), #1a86d8)',
                            color: 'white',
                            border: 'none',
                            borderRadius: '16px',
                            cursor: uploading ? 'not-allowed' : 'pointer',
                            fontWeight: 700,
                            fontSize: '15px',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '10px',
                        }}
                    >
                        {uploading ? (
                            <>
                                <div
                                    style={{
                                        width: '18px',
                                        height: '18px',
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
                                <Upload size={18} />
                                Upload
                            </>
                        )}
                    </motion.button>
                </div>
            </motion.div>

            {wallpapers.length === 0 ? (
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
                    <div style={{ fontSize: '80px', marginBottom: '20px', opacity: 0.2 }}>📁</div>
                    <h2 style={{ fontSize: '24px', fontWeight: 700, marginBottom: '12px' }}>Your collection is empty</h2>
                    <p style={{ color: 'var(--text-secondary)', fontSize: '16px', marginBottom: '28px' }}>
                        Upload your favorite wallpapers to get started
                    </p>
                    <motion.button
                        whileHover={{ scale: 1.05 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={handleUpload}
                        style={{
                            padding: '14px 28px',
                            background: 'linear-gradient(135deg, var(--accent), #1a86d8)',
                            color: 'white',
                            border: 'none',
                            borderRadius: '16px',
                            cursor: 'pointer',
                            fontWeight: 700,
                            fontSize: '15px',
                        }}
                    >
                        Upload Wallpaper
                    </motion.button>
                </motion.div>
            ) : (
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
                            thumbnail={wallpaper.thumbnail}
                            type={wallpaper.media_type === 'video' ? 'video' : 'image'}
                            isActive={isActive(wallpaper)}
                            onSet={() => handleSetWallpaper(wallpaper)}
                            onDelete={() => handleDelete(wallpaper)}
                            isVisible={true}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}
