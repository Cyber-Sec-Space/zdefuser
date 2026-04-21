import React, { useState } from 'react';
import './DropZone.css';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

interface DropZoneProps {
  onAnalyzeStarted: (path: string, password?: string) => void;
}

export const DropZone: React.FC<DropZoneProps> = ({ onAnalyzeStarted }) => {
  const [isDragActive, setIsDragActive] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [password, setPassword] = useState<string>('');

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragActive(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragActive(false);
  };

  const processFile = async (filePath: string) => {
    if (!filePath.endsWith('.zip') && !filePath.endsWith('.tar') && !filePath.endsWith('.tar.gz') && !filePath.endsWith('.tgz') && !filePath.endsWith('.rar')) {
      setError('Unsupported file type. Please use .zip, .rar, .tar, or .tgz');
      return;
    }
    
    setError(null);
    try {
      const pwdArg = password.trim() ? password.trim() : undefined;
      onAnalyzeStarted(filePath, pwdArg);
      await invoke('analyze_archive', { archivePath: filePath, password: pwdArg });
    } catch (err) {
      console.error("Failed to analyze:", err);
      setError(String(err));
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragActive(false);
  };
  
  const handleManualSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Archives',
          extensions: ['zip', 'rar', 'tar', 'tgz', 'gz']
        }]
      });
      if (typeof selected === 'string') {
        processFile(selected);
      }
    } catch (e) {
      console.error(e);
      setError(String(e));
    }
  };

  return (
    <div 
      className={`drop-zone glass-panel ${isDragActive ? 'active' : ''}`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      onClick={handleManualSelect}
    >
      <div className="drop-content">
        <div className="icon-wrapper">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
             <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v12m0-12l-4 4m4-4l4 4M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2" />
          </svg>
        </div>
        <div className="text-group">
          <h3>Select Archive to Extract</h3>
          <p>Drop file here or click to browse</p>
        </div>
        
        <div 
          className={`password-wrapper ${password ? 'has-value' : ''}`}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="input-icon">
             <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
               <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
               <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
             </svg>
          </div>
           <input 
              type="password" 
              placeholder="Archive password (optional)" 
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="password-input"
              autoComplete="off"
           />
        </div>

        {error && <div className="error-bubble">{error}</div>}
      </div>
    </div>
  );
};
