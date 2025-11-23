export type WallpaperSourceOption =
  | 'all'
  | 'picre'
  | 'wallhaven'
  | 'zerochan'
  | 'wallpapers'
  | 'moewalls'
  | 'wallpaperflare'
  | 'motionbgs';

export interface PicReImage {
  file_url: string;
  md5: string;
  tags: string[];
  width: number;
  height: number;
  source: string;
  author: string;
  has_children: boolean;
  _id: number;
}

export interface WallpaperItem {
  id: string;
  source: WallpaperSourceOption;
  title?: string;
  imageUrl: string;
  thumbnailUrl?: string;
  type?: 'image' | 'video';
  width?: number;
  height?: number;
  tags?: string[];
  metadata?: Record<string, unknown>;
  detailUrl?: string;
  original?: any;
}

