export const getSourceIcon = (source: string) => {
    const icons: Record<string, string> = {
        wallhaven: '🏔️',
        zerochan: '🎨',
        moewalls: '🌸',
        wallpapers: '🖼️',
        wallpaperflare: '🔥',
        motionbgs: '🎬',
        picre: '📸',
    };
    return icons[source] || '🖼️';
};
