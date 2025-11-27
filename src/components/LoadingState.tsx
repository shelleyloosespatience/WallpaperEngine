interface LoadingStateProps {
    text?: string;
}

export function LoadingSpinner({ text = 'Loading...' }: LoadingStateProps) {
    return (
        <div
            style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                padding: '60px 20px',
                gap: '16px',
            }}
        >
            <div
                className="spinner"
                style={{
                    width: '40px',
                    height: '40px',
                    border: '3px solid var(--border-medium)',
                    borderTop: '3px solid var(--accent)',
                    borderRadius: '50%',
                }}
            />
            <p style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>{text}</p>
        </div>
    );
}

export function SkeletonCard() {
    return (
        <div
            style={{
                width: '100%',
                aspectRatio: '16/9',
                background: 'var(--bg-tertiary)',
                borderRadius: 'var(--radius-lg)',
                animation: 'pulse 1.5s ease-in-out infinite',
            }}
        />
    );
}
