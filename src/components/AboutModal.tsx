import { useEffect } from 'react';
import licenseText from '../assets/THIRD-PARTY-NOTICES.txt?raw';
import './AboutModal.css';

interface AboutModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export function AboutModal({ isOpen, onClose }: AboutModalProps) {
    useEffect(() => {
        const handleEsc = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };
        if (isOpen) window.addEventListener('keydown', handleEsc);
        return () => window.removeEventListener('keydown', handleEsc);
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-content" onClick={e => e.stopPropagation()}>
                <div className="modal-header">
                    <h2>ZDefuser - About & Licenses</h2>
                    <button className="close-btn" onClick={onClose}>
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>
                <div className="modal-body">
                    <p className="app-version">Version 1.0.2</p>
                    <p className="app-description">
                        Zero-Trust Sandboxed Extraction for macOS and Windows. 
                        Powered by Wasmtime.
                    </p>
                    
                    <h3 className="notices-title">Third-Party Licenses</h3>
                    <div className="license-container">
                        <pre>{licenseText}</pre>
                    </div>
                </div>
            </div>
        </div>
    );
}
