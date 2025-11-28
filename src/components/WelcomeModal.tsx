import { motion, AnimatePresence } from 'framer-motion';
import { Image as ImageIcon, Video } from 'lucide-react';

interface WelcomeModalProps {
    onClose: () => void;
    onSelectType: (type: 'static' | 'live' | 'all') => void;
}

export default function WelcomeModal({ onClose, onSelectType }: WelcomeModalProps) {
    return (
        <AnimatePresence>
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                style={{
                    position: 'fixed',
                    top: 0,
                    left: 0,
                    right: 0,
                    bottom: 0,
                    background: 'rgba(0, 0, 0, 0.7)',
                    backdropFilter: 'blur(12px)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    zIndex: 9999,
                    padding: '20px',
                }}
                onClick={onClose}
            >
                <motion.div
                    initial={{ scale: 0.9, y: 20 }}
                    animate={{ scale: 1, y: 0 }}
                    exit={{ scale: 0.9, y: 20 }}
                    transition={{ type: 'spring', damping: 25 }}
                    onClick={(e) => e.stopPropagation()}
                    style={{
                        background: 'linear-gradient(135deg, rgba(30, 30, 35, 0.95), rgba(20, 20, 25, 0.95))',
                        borderRadius: '24px',
                        padding: '48px',
                        maxWidth: '520px',
                        width: '100%',
                        border: '1px solid rgba(255, 255, 255, 0.1)',
                        boxShadow: '0 24px 48px rgba(0, 0, 0, 0.5)',
                    }}
                >
                    <div style={{ textAlign: 'center', marginBottom: '32px' }}>
                        <h2 style={{ fontSize: '28px', fontWeight: 700, marginBottom: '12px' }}>
                            What are you looking for?
                        </h2>
                        <p style={{ color: 'var(--text-secondary)', fontSize: '15px' }}>
                            Choose a category to get started
                        </p>
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginBottom: '24px' }}>
                        <motion.button
                            whileHover={{ scale: 1.02 }}
                            whileTap={{ scale: 0.98 }}
                            onClick={() => onSelectType('static')}
                            style={{
                                padding: '20px 24px',
                                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                                border: 'none',
                                borderRadius: '16px',
                                color: 'white',
                                fontSize: '16px',
                                fontWeight: 600,
                                cursor: 'pointer',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '12px',
                                justifyContent: 'center',
                            }}
                        >
                            <ImageIcon size={20} />
                            Static 4k Wallpapers
                        </motion.button>

                        <motion.button
                            whileHover={{ scale: 1.02 }}
                            whileTap={{ scale: 0.98 }}
                            onClick={() => onSelectType('live')}
                            style={{
                                padding: '20px 24px',
                                background: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
                                border: 'none',
                                borderRadius: '16px',
                                color: 'white',
                                fontSize: '16px',
                                fontWeight: 600,
                                cursor: 'pointer',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '12px',
                                justifyContent: 'center',
                            }}
                        >
                            <Video size={20} />
                            Live 4k Wallpapers
                        </motion.button>

                        <motion.button
                            whileHover={{ scale: 1.02 }}
                            whileTap={{ scale: 0.98 }}
                            onClick={() => onSelectType('all')}
                            style={{
                                padding: '20px 24px',
                                background: 'rgba(255, 255, 255, 0.05)',
                                border: '1px solid rgba(255, 255, 255, 0.1)',
                                borderRadius: '16px',
                                color: 'white',
                                fontSize: '16px',
                                fontWeight: 600,
                                cursor: 'pointer',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '12px',
                                justifyContent: 'center',
                            }}
                        >
                            Nu uh You tell!
                        </motion.button>
                    </div>

                    <motion.button
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={onClose}
                        style={{
                            width: '100%',
                            padding: '14px',
                            background: 'transparent',
                            border: 'none',
                            borderRadius: '12px',
                            color: 'var(--text-secondary)',
                            fontSize: '14px',
                            fontWeight: 500,
                            cursor: 'pointer',
                        }}
                    >
                        No, thanks!
                    </motion.button>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
}
