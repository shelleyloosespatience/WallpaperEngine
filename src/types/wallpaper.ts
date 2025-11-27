export type WallpaperSourceOption = 'all' | 'wallhaven' | 'moewalls' | 'wallpapers' | 'wallpaperflare' | 'motionbgs';

export interface PicReImage {
  _id: string;
  md5: string;
  file_url: string;
  width: number;
  height: number;
  tags: string[];
  source: string;
  author: string;
  has_children: boolean;
}

export interface WallpaperItem {
  id: string;
  source: WallpaperSourceOption;
  title?: string;
  imageUrl: string;
  thumbnailUrl?: string;
  type: 'image' | 'video';
  width?: number;
  height?: number;
  tags?: string[];
  metadata?: Record<string, any>;
  detailUrl?: string;
  original?: any;
}
