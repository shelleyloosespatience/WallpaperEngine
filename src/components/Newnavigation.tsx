import { Home, Library, Settings, Store } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { useState } from 'react';

interface ModernNavigationProps {
    activeTab: string;
    onTabChange: (tab: string) => void;
}

const tabs = [
    { id: 'home', label: 'Home', icon: Home },
    { id: 'browse', label: 'Store', icon: Store },
    { id: 'library', label: 'Library', icon: Library },
    { id: 'settings', label: 'Settings', icon: Settings },
];

export default function ModernNavigation({ activeTab, onTabChange }: ModernNavigationProps) {
    const [hoveredTab, setHoveredTab] = useState<string | null>(null);

    return (
        <div
            style={{
                position: 'fixed',
                top: '48px',
                left: 0,
                width: '78px',
                height: 'calc(100vh - 48px)',
                background: 'var(--bg-secondary)',
                borderRight: '1px solid var(--border-color)',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
                borderRadius: '8px',
                padding: '16px 8px',
                zIndex: 100,
                overflow: 'hidden',
            }}
        >
            {/* glow effect js for aesthetics */}
            <AnimatePresence>
                {hoveredTab && (
                    <motion.div
                        initial={{ opacity: 0, scale: 0.8 }}
                        animate={{ opacity: 0.3, scale: 1.5 }}
                        exit={{ opacity: 0, scale: 0.8 }}
                        transition={{ duration: 0.4 }}
                        style={{
                            position: 'absolute',
                            width: '120px',
                            height: '100px',
                            background: 'radial-gradient(circle, var(--accent) 0%, transparent 87%)',
                            borderRadius: '50%',
                            pointerEvents: 'none',
                            left: '25%',
                            top: `${(tabs.findIndex(t => t.id === hoveredTab) * 70) + 50}px`,
                            transform: 'translate(-50%, -50%)',
                            filter: 'blur(20px)',
                        }}
                    />
                )}
            </AnimatePresence>

            {tabs.map((tab, index) => {
                const Icon = tab.icon;
                const isActive = activeTab === tab.id;

                return (
                    <motion.button
                        key={tab.id}
                        initial={{ x: -100, opacity: 0 }}
                        animate={{ x: 0, opacity: 1 }}
                        transition={{
                            delay: index * 0.1,
                            type: 'spring',
                            stiffness: 200,
                            damping: 20
                        }}
                        whileHover={{
                            scale: 1.08,
                            x: 4,
                            transition: { duration: 0.2 }
                        }}
                        whileTap={{
                            scale: 0.92,
                            transition: { duration: 0.1 }
                        }}
                        onHoverStart={() => setHoveredTab(tab.id)}
                        onHoverEnd={() => setHoveredTab(null)}
                        onClick={() => onTabChange(tab.id)}
                        style={{
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'center',
                            justifyContent: 'center',
                            gap: '6px',
                            padding: '12px 8px',
                            background: isActive ? 'var(--bg-tertiary)' : 'transparent',
                            border: 'none',
                            borderRadius: '12px',
                            cursor: 'pointer',
                            color: isActive ? 'var(--accent)' : 'var(--text-secondary)',
                            position: 'relative',
                            overflow: 'hidden',
                        }}
                    >
                        {/* border glow - simplified (static gradient or pulse once) */}
                        {isActive && (
                            <motion.div
                                layoutId="activeTabGlow"
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                style={{
                                    position: 'absolute',
                                    inset: 0,
                                    borderRadius: '12px',
                                    padding: '1px',
                                    background: 'linear-gradient(45deg, var(--accent), transparent, var(--accent))',
                                    WebkitMask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
                                    WebkitMaskComposite: 'xor',
                                    maskComposite: 'exclude',
                                }}
                            />
                        )}

                        {/* indicator bar */}
                        <AnimatePresence>
                            {isActive && (
                                <motion.div
                                    layoutId="activeTab"
                                    initial={{ scaleY: 0, opacity: 0 }}
                                    animate={{ scaleY: 1, opacity: 1 }}
                                    exit={{ scaleY: 0, opacity: 0 }}
                                    transition={{
                                        type: 'spring',
                                        stiffness: 300,
                                        damping: 25
                                    }}
                                    style={{
                                        position: 'absolute',
                                        left: 0,
                                        top: '50%',
                                        transform: 'translateY(-50%)',
                                        width: '3px',
                                        height: '40px',
                                        background: 'var(--accent)',
                                        borderRadius: '0 2px 2px 0',
                                        boxShadow: '0 0 10px var(--accent)',
                                    }}
                                />
                            )}
                        </AnimatePresence>

                        {/* ripple -> unstable - removed for performance */}

                        <motion.div
                            animate={isActive ? {
                                y: [0, -2, 0],
                            } : {}}
                            transition={{
                                duration: 0.5,
                                ease: 'easeOut',
                            }}
                        >
                            <Icon size={20} />
                        </motion.div>

                        <span
                            style={{
                                fontSize: '11px',
                                fontWeight: isActive ? 600 : 400,
                                position: 'relative',
                                zIndex: 1,
                            }}
                        >
                            {tab.label}
                        </span>
                    </motion.button>
                );
            })}

            {/* <motion.div
                animate={{
                    scaleX: [1, 1.2, 1],
                    opacity: [0.3, 0.6, 0.3],
                }}
                transition={{
                    duration: 3,
                    repeat: Infinity,
                    ease: 'easeInOut',
                }}
                style={{
                    position: 'absolute',
                    bottom: '16px',
                    left: '50%',
                    transform: 'translateX(-50%)',
                    width: '40px',
                    height: '2px',
                    background: 'linear-gradient(90deg, transparent, var(--accent), transparent)',
                    borderRadius: '2px',
                }}
            /> */}
        </div>
    );
}

