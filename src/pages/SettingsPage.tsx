import React from 'react';
import { invoke } from '@tauri-apps/api/core';

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
            <div style={{ padding: '24px', textAlign: 'center' }}>
                <p style={{ color: 'var(--text-secondary)' }}>Loading settings...</p>
            </div>
        );
    }

    return (
        <div style={{ padding: '24px', maxWidth: '800px' }}>
            <h1 style={{ fontSize: '24px', fontWeight: 700, marginBottom: '24px' }}>Settings</h1>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                <div className="card">
                    <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>
                        Video Wallpaper
                    </h2>

                    <div
                        style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            padding: '12px 0',
                            borderBottom: '1px solid var(--border-subtle)',
                        }}
                    >
                        <div>
                            <div style={{ fontSize: '14px', fontWeight: 500 }}>Enable Live Wallpaper</div>
                            <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '4px' }}>
                                Allow video wallpapers to be set
                            </div>
                        </div>
                        <label style={{ position: 'relative', display: 'inline-block', width: '44px', height: '24px' }}>
                            <input
                                type="checkbox"
                                checked={settings.liveWallpaperEnabled}
                                onChange={(e) => handleSaveSettings({ ...settings, liveWallpaperEnabled: e.target.checked })}
                                disabled={saving}
                                style={{ opacity: 0, width: 0, height: 0 }}
                            />
                            <span
                                style={{
                                    position: 'absolute',
                                    cursor: 'pointer',
                                    top: 0,
                                    left: 0,
                                    right: 0,
                                    bottom: 0,
                                    background: settings.liveWallpaperEnabled ? 'var(--accent)' : 'var(--border-medium)',
                                    borderRadius: '24px',
                                    transition: 'var(--transition)',
                                }}
                            >
                                <span
                                    style={{
                                        position: 'absolute',
                                        content: '',
                                        height: '18px',
                                        width: '18px',
                                        left: settings.liveWallpaperEnabled ? '23px' : '3px',
                                        bottom: '3px',
                                        background: 'white',
                                        borderRadius: '50%',
                                        transition: 'var(--transition)',
                                    }}
                                />
                            </span>
                        </label>
                    </div>

                    <div
                        style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            padding: '12px 0',
                        }}
                    >
                        <div>
                            <div style={{ fontSize: '14px', fontWeight: 500 }}>Enable Video Audio</div>
                            <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '4px' }}>
                                Play audio from video wallpapers
                            </div>
                        </div>
                        <label style={{ position: 'relative', display: 'inline-block', width: '44px', height: '24px' }}>
                            <input
                                type="checkbox"
                                checked={settings.audioEnabled}
                                onChange={(e) => handleSaveSettings({ ...settings, audioEnabled: e.target.checked })}
                                disabled={saving}
                                style={{ opacity: 0, width: 0, height: 0 }}
                            />
                            <span
                                style={{
                                    position: 'absolute',
                                    cursor: 'pointer',
                                    top: 0,
                                    left: 0,
                                    right: 0,
                                    bottom: 0,
                                    background: settings.audioEnabled ? 'var(--accent)' : 'var(--border-medium)',
                                    borderRadius: '24px',
                                    transition: 'var(--transition)',
                                }}
                            >
                                <span
                                    style={{
                                        position: 'absolute',
                                        content: '',
                                        height: '18px',
                                        width: '18px',
                                        left: settings.audioEnabled ? '23px' : '3px',
                                        bottom: '3px',
                                        background: 'white',
                                        borderRadius: '50%',
                                        transition: 'var(--transition)',
                                    }}
                                />
                            </span>
                        </label>
                    </div>

                    {videoState.isActive && (
                        <div style={{ marginTop: '12px' }}>
                            <button onClick={handleToggleLiveWallpaper} className="btn-secondary">
                                Stop Live Wallpaper
                            </button>
                        </div>
                    )}
                </div>

                <div className="card">
                    <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>Storage</h2>

                    <div
                        style={{
                            padding: '12px 0',
                            borderBottom: '1px solid var(--border-subtle)',
                        }}
                    >
                        <div style={{ fontSize: '14px', fontWeight: 500, marginBottom: '8px' }}>
                            Wallpaper Storage Path
                        </div>
                        <div
                            style={{
                                fontSize: '12px',
                                color: 'var(--text-primary)',
                                fontFamily: 'monospace',
                                background: 'var(--bg-tertiary)',
                                padding: '8px 12px',
                                borderRadius: 'var(--radius-sm)',
                                overflow: 'auto',
                            }}
                        >
                            {storagePath || 'Not available'}
                        </div>
                    </div>

                    <div style={{ padding: '12px 0' }}>
                        <div style={{ fontSize: '14px', fontWeight: 500, marginBottom: '8px' }}>
                            Cache
                        </div>
                        <div style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '12px' }}>
                            {cacheInfo.fileCount} files · {cacheInfo.sizeMB} MB
                        </div>
                        <button onClick={handleClearCache} className="btn-secondary">
                            Clear Cache
                        </button>
                    </div>
                </div>

                <div className="card">
                    <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '8px' }}>About</h2>
                    <p style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                        Colorwall v1.2.0
                    </p>
                    <p style={{ fontSize: '12px', color: 'var(--text-tertiary)', marginTop: '4px' }}>
                        Presented to you by Laxenta Inc.
                    </p>
                    <p style={{ fontSize: '15px', color: 'var(--accent)', marginTop: '4px' }}>
                        <a href="https://laxenta.tech" target="_blank" rel="noopener noreferrer">
                            https://laxenta.tech
                        </a>
                    </p>
                    <p style={{ fontSize: '21px', color: 'aqua', marginTop: '4px' }}>
                        <a href="https://github.com/shelleyloosespatience/WallpaperEngine" target="_blank" rel="noopener noreferrer">
                            Open to contributions, Click to go to our Repository.
                        </a>
                    </p>
                </div>
            </div>
        </div>
    );
}
