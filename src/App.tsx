import React from 'react';
import EnhancedTitleBar from './components/titlebar';
import ModernNavigation from './components/Newnavigation';
import SnowEffect from './components/SnowEffect';
import HomePage from './pages/HomePage';
import LibraryPage from './pages/LibraryPage';
import StorePage from './pages/StorePage';
import SettingsPage from './pages/SettingsPage';
import { AnimatePresence, motion } from 'framer-motion';

export default function App() {
    const [activeTab, setActiveTab] = React.useState('home');
    const [browsingSource, setBrowsingSource] = React.useState<string | null>(null);
    const [browsingLive, setBrowsingLive] = React.useState(false);

    const handleSourceNavigation = (source: string) => {
        setBrowsingSource(source);
        setBrowsingLive(false);
        setActiveTab('browse');
    };

    const handleLiveNavigation = () => {
        setBrowsingLive(true);
        setBrowsingSource(null);
        setActiveTab('browse');
    };

    const handleSettingsClick = () => {
        setActiveTab('settings');
        setBrowsingSource(null);
        setBrowsingLive(false);
    };

    const handleUserClick = () => {
        console.log('not yet implemented nigga, pls wait, im making this app alone');
    };

    const handleTabChange = (tab: string) => {
        setActiveTab(tab);
        if (tab !== 'browse') {
            setBrowsingSource(null);
            setBrowsingLive(false);
        }
    };

    return (
        <div style={{ minHeight: '100vh', background: 'var(--bg-primary)', position: 'relative' }}>
            <SnowEffect />
            <EnhancedTitleBar onSettingsClick={handleSettingsClick} onUserClick={handleUserClick} />
            <ModernNavigation activeTab={activeTab} onTabChange={handleTabChange} />

            <div style={{ marginLeft: '80px', marginTop: '48px', minHeight: 'calc(100vh - 48px)' }}>
                <AnimatePresence mode="wait">
                    {activeTab === 'home' && (
                        <motion.div
                            key="home"
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: 20 }}
                            transition={{ duration: 0.3 }}
                        >
                            <HomePage
                                onNavigateToSource={handleSourceNavigation}
                                onNavigateToLive={handleLiveNavigation}
                            />
                        </motion.div>
                    )}
                    {activeTab === 'browse' && (
                        <motion.div
                            key="browse"
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: 20 }}
                            transition={{ duration: 0.3 }}
                        >
                            <StorePage
                                selectedSource={browsingSource || 'all'}
                                filterType={browsingLive ? 'live' : 'static'}
                                onBack={() => setActiveTab('home')}
                            />
                        </motion.div>
                    )}
                    {activeTab === 'library' && (
                        <motion.div
                            key="library"
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: 20 }}
                            transition={{ duration: 0.3 }}
                        >
                            <LibraryPage />
                        </motion.div>
                    )}
                    {activeTab === 'settings' && (
                        <motion.div
                            key="settings"
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: 20 }}
                            transition={{ duration: 0.3 }}
                        >
                            <SettingsPage />
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </div>
    );
}
