import React from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { User, Settings } from 'lucide-react';
import { motion } from 'framer-motion';
import LaxentaLogo from './LaxentaLogo';

interface EnhancedTitleBarProps {
    onSettingsClick?: () => void;
    onUserClick?: () => void;
}

export default function EnhancedTitleBar({ onSettingsClick, onUserClick }: EnhancedTitleBarProps) {
    const [isMaximized, setIsMaximized] = React.useState(false);

    const minimize = async () => {
        const window = getCurrentWindow();
        await window.minimize();
    };

    const toggleMaximize = async () => {
        const window = getCurrentWindow();
        const maximized = await window.isMaximized();
        if (maximized) {
            await window.unmaximize();
        } else {
            await window.maximize();
        }
        setIsMaximized(!maximized);
    };

    const close = async () => {
        const window = getCurrentWindow();
        await window.close();
    };

    return (
        <div
            style={{
                position: 'fixed',
                top: 0,
                left: 0,
                right: 0,
                height: '48px',
                background: 'var(--bg-secondary)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '0 16px',
                zIndex: 9999,
                borderBottom: '1px solid var(--border-color)',
                // @ts-ignore
                WebkitAppRegion: 'drag',
            }}
        >
            <div style={{ display: 'flex', alignItems: 'center', gap: '11px', position: 'relative' }}>
                <div style={{ position: 'relative', zIndex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    {[...Array(3)].map((_, i) => (
                        <motion.div
                            key={i}
                            animate={{
                                scale: [1, 2, 1],
                                opacity: [0.5, 0, 0.5],
                            }}
                            transition={{
                                duration: 3,
                                repeat: Infinity,
                                delay: i * 1.8,
                                ease: 'easeOut',
                            }}
                            style={{
                                position: 'absolute',
                                left: '8%',
                                top: '10%',
                                transform: 'translate(-50%, -50%)',
                                width: '24px',
                                height: '24px',
                                border: '2px solid #3b82f6',
                                borderRadius: '50%',
                                pointerEvents: 'none',
                            }}
                        />
                    ))}
                    <LaxentaLogo />
                </div>
                
                <span
                    style={{ 
                        fontSize: '16px', 
                        fontWeight: 700,
                        background: 'linear-gradient(to right, #60a5fa, #818cf8)',
                        WebkitBackgroundClip: 'text',
                        WebkitTextFillColor: 'transparent',
                        backgroundClip: 'text',
                        position: 'relative',
                        zIndex: 1,
                    }}
                >
                    ColorWall
                </span>
            </div>

            <div style={{
                display: 'flex', gap: '4px',
                // @ts-ignore
                WebkitAppRegion: 'no-drag'
            }}>
                <motion.button
                    whileHover={{ backgroundColor: 'var(--bg-hover)' }}
                    whileTap={{ scale: 0.95 }}
                    onClick={onUserClick}
                    style={{
                        width: '40px',
                        height: '40px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-secondary)',
                        borderRadius: '6px',
                    }}
                >
                    <User size={18} />
                </motion.button>
                <motion.button
                    whileHover={{ backgroundColor: 'var(--bg-hover)' }}
                    whileTap={{ scale: 0.95 }}
                    onClick={onSettingsClick}
                    style={{
                        width: '40px',
                        height: '40px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-secondary)',
                        borderRadius: '6px',
                    }}
                >
                    <Settings size={18} />
                </motion.button>
                <div style={{ width: '1px', height: '24px', background: 'var(--border-color)', margin: '0 8px' }} />
                <motion.button
                    whileHover={{ backgroundColor: 'var(--bg-hover)' }}
                    whileTap={{ scale: 0.95 }}
                    onClick={minimize}
                    style={{
                        width: '46px',
                        height: '40px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-secondary)',
                        fontSize: '16px',
                    }}
                >
                    ─
                </motion.button>
                <motion.button
                    whileHover={{ backgroundColor: 'var(--bg-hover)' }}
                    whileTap={{ scale: 0.95 }}
                    onClick={toggleMaximize}
                    style={{
                        width: '46px',
                        height: '40px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-secondary)',
                        fontSize: '16px',
                    }}
                >
                    {isMaximized ? '❐' : '☐'}
                </motion.button>
                <motion.button
                    whileHover={{ backgroundColor: '#e81123' }}
                    whileTap={{ scale: 0.95 }}
                    onClick={close}
                    style={{
                        width: '46px',
                        height: '40px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-secondary)',
                        fontSize: '16px',
                    }}
                >
                    ✕
                </motion.button>
            </div>
        </div>
    );
}