import { motion } from 'framer-motion';

export default function LoadingScreen() {
    return (
        <motion.div
            initial={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.5 }}
            style={{
                position: 'fixed',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                background: 'linear-gradient(135deg, #1a1a1f 0%, #0f0f14 100%)',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                zIndex: 10000,
                overflow: 'hidden',
            }}
        >
            {/* logo container */}
            <motion.div
                initial={{ x: -100, opacity: 0 }}
                animate={{ x: 0, opacity: 1 }}
                transition={{
                    duration: 0.8,
                    ease: [0.22, 1, 0.36, 1],
                }}
                style={{
                    position: 'relative',
                    marginBottom: '40px',
                }}
            >
                <motion.div
                    animate={{ rotate: 360 }}
                    transition={{
                        duration: 6,
                        repeat: Infinity,
                        ease: 'linear',
                    }}
                    style={{
                        position: 'absolute',
                        top: '-25px',
                        left: '-25px',
                        right: '-25px',
                        bottom: '-25px',
                        borderRadius: '50%',
                        background: 'conic-gradient(from 0deg, transparent, rgba(139, 92, 246, 0.3), rgba(0, 120, 212, 0.2), transparent, transparent)',
                        opacity: 0.4,
                        filter: 'blur(10px)',
                    }}
                />
                {/* top */}
                <motion.div
                    animate={{ x: ['0%', '400%'] }}
                    transition={{
                        duration: 5.2,
                        repeat: Infinity,
                        ease: 'linear',
                    }}
                    style={{
                        position: 'absolute',
                        top: '0px',
                        left: '0px',
                        width: '30px',
                        height: '2px',
                        background: 'linear-gradient(90deg, transparent, rgba(0, 217, 255, 0.6), transparent)',
                        borderRadius: '1px',
                    }}
                />

                {/* Right */}
                <motion.div
                    animate={{ y: ['0%', '400%'] }}
                    transition={{
                        duration: 4.2,
                        repeat: Infinity,
                        ease: 'linear',
                        delay: 0.375,
                    }}
                    style={{
                        position: 'absolute',
                        top: '0px',
                        right: '0px',
                        width: '2px',
                        height: '30px',
                        background: 'linear-gradient(180deg, transparent, rgba(0, 217, 255, 0.6), transparent)',
                        borderRadius: '1px',
                    }}
                />

                {/* bottom */}
                <motion.div
                    animate={{ x: ['0%', '-400%'] }}
                    transition={{
                        duration: 4.2,
                        repeat: Infinity,
                        ease: 'linear',
                        delay: 0.65,
                    }}
                    style={{
                        position: 'absolute',
                        bottom: '0px',
                        right: '0px',
                        width: '30px',
                        height: '2px',
                        background: 'linear-gradient(270deg, transparent, rgba(0, 217, 255, 0.6), transparent)',
                        borderRadius: '1px',
                    }}
                />

                {/* Left */}
                <motion.div
                    animate={{ y: ['0%', '-400%'] }}
                    transition={{
                        duration: 3.2,
                        repeat: Infinity,
                        ease: 'linear',
                        delay: 0.75,
                    }}
                    style={{
                        position: 'absolute',
                        bottom: '0px',
                        left: '0px',
                        width: '2px',
                        height: '30px',
                        background: 'linear-gradient(0deg, transparent, rgba(0, 217, 255, 0.6), transparent)',
                        borderRadius: '1px',
                    }}
                />

                {/* Actual Logo Image with subtle drop-shadow */}
                <motion.img
                    src="/Square150x150Logo.png"
                    alt="Colorwall"
                    initial={{ scale: 0.9 }}
                    animate={{
                        scale: 1,
                        filter: [
                            'drop-shadow(0 0 6px rgba(139, 92, 246, 0.3)) drop-shadow(0 0 12px rgba(0, 120, 212, 0.2))',
                            'drop-shadow(0 0 10px rgba(139, 92, 246, 0.4)) drop-shadow(0 0 20px rgba(0, 120, 212, 0.25))',
                            'drop-shadow(0 0 6px rgba(139, 92, 246, 0.3)) drop-shadow(0 0 12px rgba(0, 120, 212, 0.2))',
                        ],
                    }}
                    transition={{
                        scale: { duration: 0.5, delay: 0.3 },
                        filter: { duration: 2.5, repeat: Infinity, ease: 'easeInOut' },
                    }}
                    style={{
                        width: '150px',
                        height: '150px',
                        position: 'relative',
                        zIndex: 2,
                    }}
                />
            </motion.div>

            {/* App Name */}
            <motion.h1
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.4, duration: 0.5 }}
                style={{
                    fontSize: '42px',
                    fontWeight: 800,
                    background: 'linear-gradient(135deg, #0078d4, #00d9ff)',
                    WebkitBackgroundClip: 'text',
                    WebkitTextFillColor: 'transparent',
                    marginBottom: '16px',
                    letterSpacing: '-0.02em',
                }}
            >
                Colorwall
            </motion.h1>

            {/* Loading Text with Animated Dots */}
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.6, duration: 0.5 }}
                style={{
                    color: 'var(--text-secondary)',
                    fontSize: '15px',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                }}
            >
                <span>Initializing</span>
                <motion.div style={{ display: 'flex', gap: '4px' }}>
                    {[0, 1, 2].map((i) => (
                        <motion.div
                            key={i}
                            animate={{
                                opacity: [0.3, 1, 0.3],
                            }}
                            transition={{
                                duration: 1.5,
                                repeat: Infinity,
                                delay: i * 0.2,
                            }}
                            style={{
                                width: '6px',
                                height: '6px',
                                borderRadius: '50%',
                                background: 'var(--accent)',
                            }}
                        />
                    ))}
                </motion.div>
            </motion.div>
        </motion.div>
    );
}
