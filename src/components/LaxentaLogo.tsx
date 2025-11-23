import React from 'react';

const LaxentaLogo = () => (
  <div className="relative w-8 h-8">
    <div className="absolute inset-0 rounded-full border-2 border-blue-500/40 animate-soundwave" />
    <div className="absolute inset-0 rounded-full border-2 border-blue-400/30 animate-soundwave" style={{ animationDelay: '0.5s' }} />
    <div className="absolute inset-0 rounded-full border-2 border-indigo-500/30 animate-soundwave" style={{ animationDelay: '1s' }} />
    <div className="absolute inset-0 rounded-full border-2 border-indigo-400/20 animate-soundwave" style={{ animationDelay: '1.5s' }} />

    <div className="relative w-8 h-8 rounded-full overflow-hidden border-2 border-blue-500/50 z-10">
      <img
        src="/128x128.png"
        alt="ColorWall uwugo"
        className="w-full h-full object-cover"
        onError={(e: React.SyntheticEvent<HTMLImageElement>) => {
          e.currentTarget.style.display = 'none';
          e.currentTarget.parentElement!.setAttribute(
            'style',
            'background: linear-gradient(135deg, #000, #000);'
          );
        }}
      />
    </div>
  </div>
);

export default LaxentaLogo;

