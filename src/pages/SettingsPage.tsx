// this page will soon a lot of settings, let me get the app stable first :>
import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { Settings, Video, Volume2, HardDrive, Trash2, Info, Folder } from 'lucide-react';

interface AppSettings {
    audioEnabled: boolean;
    liveWallpaperEnabled: boolean;
}

export default function SettingsPage() {
    const [settings, setSettings] = React.useState<AppSettings>({
        audioEnabled: false,
        liveWallpaperEnabled: true,
    });
    const [storagePath, setStoragePath] = React.useState('');
    const [cacheInfo, setCacheInfo] = React.useState({ sizeMB: '0', fileCount: 0 });
    const [videoState, setVideoState] = React.useState({ isActive: false });
    const [loading, setLoading] = React.useState(true);
    const [saving, setSaving] = React.useState(false);

    const loadData = React.useCallback(async () => {
        try {
            const [settingsRes, pathRes, cache, video]: any = await Promise.all([
                invoke('get_settings'),
                invoke('get_wallpaper_storage_path'),
                invoke('get_cache_size'),
                invoke('get_video_wallpaper_status'),
            ]);

            if (settingsRes.success && settingsRes.settings) {
                setSettings(settingsRes.settings);
            }

            if (pathRes.success && pathRes.path) {
                setStoragePath(pathRes.path);
            }

            if (cache.success) {
                setCacheInfo({ sizeMB: cache.sizeMb, fileCount: cache.fileCount });
            }

            if (video) {
                setVideoState(video);
            }
        } catch (error) {
            console.error('Failed to load settings:', error);
        } finally {
            setLoading(false);
        }
    }, []);

    React.useEffect(() => {
        loadData();
    }, [loadData]);

    const handleSaveSettings = async (newSettings: AppSettings) => {
        setSaving(true);
        try {
            const result: any = await invoke('save_settings', { settings: newSettings });

            if (result.success) {
                setSettings(newSettings);
            } else {
                alert('Failed to save settings: ' + result.error);
            }
        } catch (error) {
            console.error('Save failed:', error);
            alert('Error: ' + error);
        } finally {
            setSaving(false);
        }
    };

    const handleToggleLiveWallpaper = async () => {
        if (videoState.isActive) {
            try {
                const result: any = await invoke('stop_video_wallpaper_command');
                if (result.success) {
                    setVideoState({ isActive: false });
                }
            } catch (error) {
                console.error('Failed to stop wallpaper:', error);
            }
        }
    };

    const handleClearCache = async () => {
        if (!confirm('Clear cache? This will delete all downloaded wallpapers.')) return;

        try {
            const result: any = await invoke('clear_cache');
            if (result.success) {
                alert(`Cache cleared! ${result.filesDeleted} files deleted.`);
                await loadData();
            }
        } catch (error) {
            console.error('Clear cache failed:', error);
            alert('Error: ' + error);
        }
    };

    if (loading) {
        return (
            <div style={{
                padding: '48px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                minHeight: '50vh'
            }}>
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    style={{ textAlign: 'center' }}
                >
                    <div
                        style={{
                            width: '48px',
                            height: '48px',
                            border: '4px solid var(--border-medium)',
                            borderTop: '4px solid var(--accent)',
                            borderRadius: '50%',
                            animation: 'spin 0.8s linear infinite',
                            margin: '0 auto 16px',
                        }}
                    />
                    <p style={{ color: 'var(--text-secondary)', fontSize: '15px' }}>Loading settings...</p>
                </motion.div>
            </div>
        );
    }

    return (
        <div style={{ padding: '48px 40px', maxWidth: '900px', margin: '0 auto' }}>
            {/* head */}
            <motion.div
                initial={{ y: -30, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ duration: 0.5 }}
                style={{ marginBottom: '40px' }}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '12px' }}>
                    <Settings size={32} style={{ color: 'var(--accent)' }} />
                    <h1 style={{
                        fontSize: '36px',
                        fontWeight: 800,
                        background: 'linear-gradient(135deg, var(--accent), var(--accent-hover))',
                        WebkitBackgroundClip: 'text',
                        WebkitTextFillColor: 'transparent',
                        letterSpacing: '-0.02em'
                    }}>
                        Settings
                    </h1>
                </div>
                <p style={{ color: 'var(--text-secondary)', fontSize: '15px' }}>
                    Configure your wallpaper engine preferences
                </p>
            </motion.div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '48px' }}>
                {/* vid Wallpaper Settings */}
                <motion.div
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    transition={{ delay: 0.1, duration: 0.5 }}
                >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '24px' }}>
                        <Video size={24} style={{ color: 'var(--accent)' }} />
                        <h2 style={{ fontSize: '20px', fontWeight: 700 }}>
                            Video Wallpaper
                        </h2>
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                        {/* Wallpaper Toggle */}
                        <div
                            style={{
                                display: 'flex',
                                justifyContent: 'space-between',
                                alignItems: 'center',
                                padding: '16px',
                                background: 'rgba(0, 0, 0, 0.2)',
                                borderRadius: 'var(--radius-md)',
                                border: '1px solid var(--border-subtle)',
                                transition: 'var(--transition)',
                            }}
                        >
                            <div style={{ flex: 1 }}>
                                <div style={{ fontSize: '15px', fontWeight: 600, marginBottom: '6px' }}>
                                    Enable Live Wallpaper
                                </div>
                                <div style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                                    Allow video wallpapers to be set on your desktop
                                </div>
                            </div>
                            <label className="toggle-switch">
                                <input
                                    type="checkbox"
                                    checked={settings.liveWallpaperEnabled}
                                    onChange={(e) => handleSaveSettings({ ...settings, liveWallpaperEnabled: e.target.checked })}
                                    disabled={saving}
                                />
                                <span className="toggle-slider" />
                            </label>
                        </div>

                        {/* audio Toggle */}
                        <div
                            style={{
                                display: 'flex',
                                justifyContent: 'space-between',
                                alignItems: 'center',
                                padding: '16px',
                                background: 'rgba(0, 0, 0, 0.2)',
                                borderRadius: 'var(--radius-md)',
                                border: '1px solid var(--border-subtle)',
                                transition: 'var(--transition)',
                            }}
                        >
                            <div style={{ flex: 1 }}>
                                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px' }}>
                                    <Volume2 size={16} style={{ color: 'var(--accent)' }} />
                                    <div style={{ fontSize: '15px', fontWeight: 600 }}>
                                        Enable Video Audio
                                    </div>
                                </div>
                                <div style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                                    Play audio from some video wallpapers (not all have audio)
                                </div>
                            </div>
                            <label className="toggle-switch">
                                <input
                                    type="checkbox"
                                    checked={settings.audioEnabled}
                                    onChange={(e) => handleSaveSettings({ ...settings, audioEnabled: e.target.checked })}
                                    disabled={saving}
                                />
                                <span className="toggle-slider" />
                            </label>
                        </div>

                        {/* stop */}
                        {videoState.isActive && (
                            <motion.div
                                initial={{ opacity: 0, height: 0 }}
                                animate={{ opacity: 1, height: 'auto' }}
                                exit={{ opacity: 0, height: 0 }}
                                style={{
                                    padding: '16px',
                                    background: 'linear-gradient(135deg, rgba(220, 38, 38, 0.1), rgba(185, 28, 28, 0.05))',
                                    borderRadius: 'var(--radius-md)',
                                    border: '1px solid rgba(220, 38, 38, 0.2)',
                                }}
                            >
                                <div style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '12px' }}>
                                    A video wallpaper is currently active
                                </div>
                                <button onClick={handleToggleLiveWallpaper} className="btn-danger">
                                    Stop Live Wallpaper
                                </button>
                            </motion.div>
                        )}
                    </div>
                </motion.div>

                {/* Divider */}
                <div style={{ height: '1px', background: 'var(--border-subtle)' }} />

                {/* Storage Settings */}
                <motion.div
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    transition={{ delay: 0.2, duration: 0.5 }}
                >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '24px' }}>
                        <HardDrive size={24} style={{ color: 'var(--accent)' }} />
                        <h2 style={{ fontSize: '20px', fontWeight: 700 }}>Storage</h2>
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                        {/* Storage Path */}
                        <div>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
                                <Folder size={16} style={{ color: 'var(--text-secondary)' }} />
                                <div style={{ fontSize: '14px', fontWeight: 600 }}>
                                    Wallpaper Storage Path
                                </div>
                            </div>
                            <div
                                style={{
                                    fontSize: '13px',
                                    color: 'var(--text-primary)',
                                    fontFamily: 'Consolas, Monaco, monospace',
                                    background: 'rgba(0, 0, 0, 0.3)',
                                    padding: '12px 16px',
                                    borderRadius: 'var(--radius-md)',
                                    border: '1px solid var(--border-subtle)',
                                    overflow: 'auto',
                                    wordBreak: 'break-all',
                                }}
                            >
                                {storagePath || 'Not available'}
                            </div>
                        </div>

                        {/* Cache Info */}
                        <div
                            style={{
                                padding: '16px',
                                background: 'rgba(0, 0, 0, 0.2)',
                                borderRadius: 'var(--radius-md)',
                                border: '1px solid var(--border-subtle)',
                            }}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
                                <Trash2 size={16} style={{ color: 'var(--text-secondary)' }} />
                                <div style={{ fontSize: '14px', fontWeight: 600 }}>
                                    Cache Management
                                </div>
                            </div>
                            <div style={{
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'space-between',
                                marginBottom: '12px'
                            }}>
                                <div>
                                    <div style={{ fontSize: '24px', fontWeight: 700, color: 'var(--accent)' }}>
                                        {cacheInfo.sizeMB} MB
                                    </div>
                                    <div style={{ fontSize: '12px', color: 'var(--text-tertiary)' }}>
                                        {cacheInfo.fileCount} files cached
                                    </div>
                                </div>
                                <button onClick={handleClearCache} className="btn-secondary">
                                    Clear Cache
                                </button>
                            </div>
                        </div>
                    </div>
                </motion.div>

                {/* Divider */}
                <div style={{ height: '1px', background: 'var(--border-subtle)' }} />

                {/* About Section */}
                <motion.div
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    transition={{ delay: 0.3, duration: 0.5 }}
                >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '24px' }}>
                        <Info size={24} style={{ color: 'var(--accent)' }} />
                        <h2 style={{ fontSize: '20px', fontWeight: 700 }}>About</h2>
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                        {/* App Info */}
                        <div
                            style={{
                                padding: '20px',
                                background: 'linear-gradient(135deg, rgba(0, 120, 212, 0.15), rgba(26, 134, 216, 0.08))',
                                borderRadius: 'var(--radius-md)',
                                border: '1px solid rgba(0, 120, 212, 0.25)',
                            }}
                        >
                            <div style={{ fontSize: '32px', fontWeight: 800, marginBottom: '8px', letterSpacing: '-0.02em' }}>
                                Colorwall
                            </div>
                            <div style={{ fontSize: '15px', color: 'var(--text-secondary)', marginBottom: '12px' }}>
                                Version 1.2.0
                            </div>
                            <div style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>
                                Presented to you by <span style={{ color: 'var(--accent)', fontWeight: 600 }}>Laxenta Inc.</span>
                            </div>
                            <a
                                href="https://laxenta.tech"
                                target="_blank"
                                rel="noopener noreferrer"
                                style={{
                                    display: 'inline-block',
                                    marginTop: '12px',
                                    color: 'var(--accent)',
                                    fontSize: '14px',
                                    textDecoration: 'none',
                                    fontWeight: 600,
                                    transition: 'color 0.2s ease',
                                }}
                                onMouseOver={(e) => e.currentTarget.style.color = 'var(--accent-hover)'}
                                onMouseOut={(e) => e.currentTarget.style.color = 'var(--accent)'}
                            >
                                 Visit My Website →
                            </a>
                        </div>

                        {/* Repository CTA - Prominent */}
                        <div
                            style={{
                                padding: '24px',
                                background: 'linear-gradient(135deg, rgba(0, 217, 255, 0.15), rgba(0, 255, 255, 0.08))',
                                borderRadius: 'var(--radius-md)',
                                border: '2px solid rgba(0, 217, 255, 0.4)',
                                boxShadow: '0 4px 20px rgba(0, 217, 255, 0.15)',
                            }}
                        >
                            <div style={{
                                fontSize: '20px',
                                fontWeight: 700,
                                marginBottom: '12px',
                                color: '#00ffff',
                                letterSpacing: '-0.01em',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '10px'
                            }}>
                                <span style={{ fontSize: '24px' }}>⭐</span>
                                <span>Open Source & Contributions Welcome</span>
                            </div>
                            <p style={{
                                fontSize: '14px',
                                color: 'var(--text-secondary)',
                                marginBottom: '16px',
                                lineHeight: '1.6'
                            }}>
                                This project is open source and we welcome contributions from the community.
                                Help us make Colorwall even better!
                            </p>
                            <a
                                href="https://github.com/shelleyloosespatience/WallpaperEngine"
                                target="_blank"
                                rel="noopener noreferrer"
                                style={{
                                    display: 'inline-block',
                                    padding: '12px 24px',
                                    background: 'linear-gradient(135deg, #00d9ff, #00ffff)',
                                    color: '#0a0a0a',
                                    fontSize: '15px',
                                    fontWeight: 700,
                                    textDecoration: 'none',
                                    borderRadius: 'var(--radius-md)',
                                    transition: 'all 0.2s ease',
                                    boxShadow: '0 4px 12px rgba(0, 217, 255, 0.3)',
                                    fontFamily: 'Segoe UI, system-ui, sans-serif',
                                }}
                                onMouseOver={(e) => {
                                    e.currentTarget.style.transform = 'translateY(-2px)';
                                    e.currentTarget.style.boxShadow = '0 6px 20px rgba(0, 217, 255, 0.4)';
                                }}
                                onMouseOut={(e) => {
                                    e.currentTarget.style.transform = 'translateY(0)';
                                    e.currentTarget.style.boxShadow = '0 4px 12px rgba(0, 217, 255, 0.3)';
                                }}
                            >
                                🚀 Visit GitHub Repository →
                            </a>
                        </div>
                    </div>
                </motion.div>
            </div>
        </div>
    );
}
