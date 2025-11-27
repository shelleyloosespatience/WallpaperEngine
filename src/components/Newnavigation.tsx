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
                width: '80px',
                height: 'calc(100vh - 48px)',
                background: 'var(--bg-secondary)',
                borderRight: '1px solid var(--border-color)',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
                padding: '16px 8px',
                zIndex: 100,
                overflow: 'hidden',
            }}
        >
            {/* Ambient glow effect */}
            <AnimatePresence>
                {hoveredTab && (
                    <motion.div
                        initial={{ opacity: 0, scale: 0.8 }}
                        animate={{ opacity: 0.3, scale: 1.5 }}
                        exit={{ opacity: 0, scale: 0.8 }}
                        transition={{ duration: 0.4 }}
                        style={{
                            position: 'absolute',
                            width: '100px',
                            height: '100px',
                            background: 'radial-gradient(circle, var(--accent) 0%, transparent 70%)',
                            borderRadius: '50%',
                            pointerEvents: 'none',
                            left: '50%',
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
                const isHovered = hoveredTab === tab.id;

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
                        {/* Animated border glow */}
                        {isActive && (
                            <motion.div
                                initial={{ scale: 0.8, opacity: 0 }}
                                animate={{ scale: 1, opacity: 1 }}
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
                            >
                                <motion.div
                                    animate={{
                                        rotate: 360,
                                    }}
                                    transition={{
                                        duration: 3,
                                        repeat: Infinity,
                                        ease: 'linear',
                                    }}
                                    style={{
                                        position: 'absolute',
                                        inset: -2,
                                        background: 'conic-gradient(from 0deg, transparent, var(--accent), transparent)',
                                        borderRadius: '12px',
                                    }}
                                />
                            </motion.div>
                        )}

                        {/* Active indicator bar */}
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

                        {/* Ripple effect on hover */}
                        <AnimatePresence>
                            {isHovered && !isActive && (
                                <>
                                    <motion.div
                                        initial={{ scale: 0, opacity: 0.5 }}
                                        animate={{ scale: 2, opacity: 0 }}
                                        exit={{ scale: 0, opacity: 0 }}
                                        transition={{ duration: 0.6, repeat: Infinity }}
                                        style={{
                                            position: 'absolute',
                                            width: '40px',
                                            height: '40px',
                                            borderRadius: '50%',
                                            border: '2px solid var(--accent)',
                                        }}
                                    />
                                    <motion.div
                                        initial={{ scale: 0, opacity: 0.5 }}
                                        animate={{ scale: 2, opacity: 0 }}
                                        exit={{ scale: 0, opacity: 0 }}
                                        transition={{ duration: 0.6, delay: 0.2, repeat: Infinity }}
                                        style={{
                                            position: 'absolute',
                                            width: '40px',
                                            height: '40px',
                                            borderRadius: '50%',
                                            border: '2px solid var(--accent)',
                                        }}
                                    />
                                </>
                            )}
                        </AnimatePresence>

                        {/* Icon with floating animation */}
                        <motion.div
                            animate={isActive ? {
                                y: [0, -3, 0],
                                rotate: [0, 5, -5, 0],
                            } : {}}
                            transition={{
                                duration: 2,
                                repeat: isActive ? Infinity : 0,
                                ease: 'easeInOut',
                            }}
                        >
                            <Icon size={20} />
                        </motion.div>

                        {/* Label with stagger effect */}
                        <motion.span
                            animate={isActive ? {
                                scale: [1, 1.05, 1],
                            } : {}}
                            transition={{
                                duration: 2,
                                repeat: isActive ? Infinity : 0,
                                ease: 'easeInOut',
                            }}
                            style={{
                                fontSize: '11px',
                                fontWeight: isActive ? 600 : 400,
                                position: 'relative',
                                zIndex: 1,
                            }}
                        >
                            {tab.label}
                        </motion.span>

                        {/* Particle burst on active */}
                        {isActive && (
                            <>
                                {[...Array(6)].map((_, i) => (
                                    <motion.div
                                        key={i}
                                        initial={{ scale: 0, x: 0, y: 0, opacity: 0 }}
                                        animate={{
                                            scale: [0, 1, 0],
                                            x: Math.cos((i * Math.PI * 2) / 6) * 30,
                                            y: Math.sin((i * Math.PI * 2) / 6) * 30,
                                            opacity: [0, 0.6, 0],
                                        }}
                                        transition={{
                                            duration: 2,
                                            repeat: Infinity,
                                            delay: i * 0.1,
                                            ease: 'easeOut',
                                        }}
                                        style={{
                                            position: 'absolute',
                                            width: '4px',
                                            height: '4px',
                                            borderRadius: '50%',
                                            background: 'var(--accent)',
                                            pointerEvents: 'none',
                                        }}
                                    />
                                ))}
                            </>
                        )}
                    </motion.button>
                );
            })}

            {/* Bottom decorative element */}
            <motion.div
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
            />
        </div>
    );
}

