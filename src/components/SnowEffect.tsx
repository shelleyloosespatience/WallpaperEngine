import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';

interface Snowflake {
    id: number;
    x: number;
    size: number;
    duration: number;
    delay: number;
}

export default function SnowEffect() {
    const [snowflakes, setSnowflakes] = useState<Snowflake[]>([]);

    useEffect(() => {
        const flakes: Snowflake[] = Array.from({ length: 50 }, (_, i) => ({
            id: i,
            x: Math.random() * 100,
            size: Math.random() * 3 + 2,
            duration: Math.random() * 10 + 10,
            delay: Math.random() * 5,
        }));
        setSnowflakes(flakes);
    }, []);

    return (
        <div
            style={{
                position: 'fixed',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                pointerEvents: 'none',
                zIndex: 9998,
                overflow: 'hidden',
            }}
        >
            {snowflakes.map((flake) => (
                <motion.div
                    key={flake.id}
                    initial={{ y: -20, x: `${flake.x}vw`, opacity: 0 }}
                    animate={{
                        y: '100vh',
                        x: [`${flake.x}vw`, `${flake.x + (Math.random() - 0.5) * 10}vw`, `${flake.x}vw`],
                        opacity: [0, 0.8, 0.8, 0],
                    }}
                    transition={{
                        duration: flake.duration,
                        delay: flake.delay,
                        repeat: Infinity,
                        ease: 'linear',
                    }}
                    style={{
                        position: 'absolute',
                        width: `${flake.size}px`,
                        height: `${flake.size}px`,
                        background: 'rgba(255, 255, 255, 0.8)',
                        borderRadius: '50%',
                        boxShadow: '0 0 10px rgba(255, 255, 255, 0.5)',
                    }}
                />
            ))}
        </div>
    );
}
