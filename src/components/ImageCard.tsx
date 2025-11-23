import React, { useEffect, useRef, useState } from 'react';
import { Image, Play, ZoomIn } from 'lucide-react';
import { WallpaperItem } from '../types/wallpaper';
import { getSourceIcon } from './icons';

interface ImageCardProps {
  image: WallpaperItem;
  onSelect: () => void;
  isVisible: boolean;
}

const ImageCard = ({ image, onSelect, isVisible }: ImageCardProps) => {
  const [imgLoaded, setImgLoaded] = useState(false);
  const [imgError, setImgError] = useState(false);
  const [shouldLoad, setShouldLoad] = useState(false);
  const imgRef = useRef<HTMLImageElement>(null);
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isVisible) {
      setShouldLoad(false);
      setImgLoaded(false);
      if (imgRef.current) {
        imgRef.current.src = '';
      }
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setShouldLoad(true);
        }
      },
      { rootMargin: '200px' }
    );

    if (cardRef.current) {
      observer.observe(cardRef.current);
    }

    return () => observer.disconnect();
  }, [isVisible]);

  useEffect(() => {
    if (shouldLoad && imgRef.current && !imgRef.current.src) {
      const src = image.thumbnailUrl || image.imageUrl;
      imgRef.current.src = src;
    }
  }, [shouldLoad, image]);

  return (
    <div
      ref={cardRef}
      className="group relative bg-black/70 rounded-xl overflow-hidden border border-gray-900/50 hover:border-blue-500/30 transition-all duration-300 cursor-pointer shadow-lg hover:shadow-xl hover:shadow-blue-500/10 backdrop-blur-sm animate-fadeIn break-inside-avoid mb-4"
      onClick={onSelect}
      style={{ height: 'auto' }}
    >
      <div className="relative overflow-hidden bg-black flex items-center justify-center" style={{ height: 'auto' }}>
        {!imgLoaded && !imgError && shouldLoad && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="w-6 h-6 border-2 border-blue-500/20 border-t-blue-500 rounded-full animate-spin" />
          </div>
        )}

        {shouldLoad ? (
          <img
            ref={imgRef}
            alt={image.title ?? `Wallpaper ${image.id}`}
            className={`w-full h-auto object-contain group-hover:scale-105 transition-all duration-500 ${
              imgLoaded ? 'opacity-100' : 'opacity-0'
            }`}
            loading="lazy"
            onLoad={() => setImgLoaded(true)}
            onError={() => setImgError(true)}
            onContextMenu={(e: React.MouseEvent) => {
              e.preventDefault();
              return false;
            }}
            draggable={false}
          />
        ) : (
          <div className="absolute inset-0 bg-gray-950 flex items-center justify-center">
            <div className="w-6 h-6 border-2 border-gray-800 border-t-gray-700 rounded-full animate-spin" />
          </div>
        )}

        {imgError && (
          <div className="absolute inset-0 flex items-center justify-center text-gray-700">
            <div className="text-center">
              <Image className="w-6 h-6 mx-auto mb-2 opacity-50" />
              <span className="text-xs">Failed to load</span>
            </div>
          </div>
        )}

        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/10 to-transparent opacity-0 group-hover:opacity-100 transition-all duration-300 flex items-center justify-center">
          {image.type === 'video' && image.source === 'motionbgs' ? (
            <div className="flex items-center gap-2 bg-emerald-500/90 px-4 py-2 rounded-full text-sm font-semibold">
              <Play className="w-4 h-4" />
              Motion BG
            </div>
          ) : image.type === 'video' ? (
            <div className="flex items-center gap-2 bg-purple-500/90 px-4 py-2 rounded-full text-sm font-semibold">
              <Play className="w-4 h-4" />
              View Live2D
            </div>
          ) : (
            <div className="flex items-center gap-2 bg-blue-500/90 px-4 py-2 rounded-full text-sm font-semibold">
              <ZoomIn className="w-4 h-4" />
              View Full Size
            </div>
          )}
        </div>
      </div>

      <div className="p-3 bg-black/90 backdrop-blur-sm">
        <div className="flex items-center justify-between mb-1.5">
          <span className="inline-flex items-center gap-1.5 text-xs font-bold uppercase tracking-wider text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded">
            {getSourceIcon(image.source)}
            <span>{image.source}</span>
          </span>
          {image.width && image.height && (
            <span className="text-xs text-gray-600 font-mono">
              {image.width}×{image.height}
            </span>
          )}
        </div>
        <p className="text-xs text-gray-400 truncate" title={image.title}>
          {image.title || 'Untitled'}
        </p>
      </div>
    </div>
  );
};

export default ImageCard;

