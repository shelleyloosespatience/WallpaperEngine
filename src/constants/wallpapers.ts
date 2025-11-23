import { WallpaperSourceOption } from '../types/wallpaper';

export const API_BASE_URL = 'https://pic.re';
export const DEFAULT_FETCH_COUNT = 20;
export const MAX_INPUT_LENGTH = 100;
export const UNLOAD_THRESHOLD = 5;

export const SOURCE_OPTIONS: { value: WallpaperSourceOption; label: string }[] = [
  { value: 'all', label: 'ALL' },
  { value: 'wallhaven', label: 'Wallhaven' },
  { value: 'zerochan', label: 'Zerochan' },
  { value: 'wallpapers', label: 'Wallpapers' },
  { value: 'moewalls', label: 'Moewalls' },
  { value: 'wallpaperflare', label: 'WPFlare' },
  { value: 'picre', label: 'pic.re' },
  { value: 'motionbgs', label: 'Live Wallpapers' },
];

