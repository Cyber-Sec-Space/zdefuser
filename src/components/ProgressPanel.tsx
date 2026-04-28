import React, { useRef, useEffect } from 'react';
import './ProgressPanel.css';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export type SandboxEvent = 
  | { type: 'progress', current: number, total: number, file: string, bytes: number }
  | { type: 'warning', code: string, file: string, details: string }
  | { type: 'complete', files_extracted: number, files_blocked: number, total_bytes: number }
  | { type: 'error', code: string, details: string };

interface ProgressPanelProps {
  events: SandboxEvent[];
  isComplete: boolean;
  hasError: boolean;
  onReset: () => void;
}

const formatBytes = (bytes: number, decimals = 2) => {
  if (!+bytes) return '0 Bytes';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
};

export const ProgressPanel: React.FC<ProgressPanelProps> = ({ events, isComplete, hasError, onReset }) => {
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [events]);

  const handleSaveTo = async () => {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: "Select Extraction Destination"
      });
      if (selectedPath) {
        await invoke('release_files', { targetDir: selectedPath });
        alert("Extraction Completed and Saved!");
      }
    } catch (e) {
      console.error(e);
      alert("Failed to save: " + e);
    }
  };

  const getProgressPercentage = () => {
    for (let i = events.length - 1; i >= 0; i--) {
      const e = events[i];
      if (e.type === 'progress' && e.total > 0) {
        return Math.min(100, Math.round((e.current / e.total) * 100));
      }
      if (e.type === 'complete') return 100;
    }
    return isComplete ? 100 : undefined;
  };

  const percentage = getProgressPercentage();

  return (
    <div className="progress-panel">
      <div className="panel-header">
        <div className="title-group">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="4 17 10 11 4 5"></polyline>
            <line x1="12" y1="19" x2="20" y2="19"></line>
          </svg>
          <div className="panel-title">Sandbox Execution</div>
        </div>
        <div className="status-indicator">
          <div className={`dot ${hasError ? 'error' : isComplete ? 'success' : 'scanning'}`}></div>
          <span>{hasError ? 'Blocked' : isComplete ? 'Verified' : 'Processing'}</span>
          {!isComplete && !hasError && (
            <span style={{ opacity: 0.7, marginLeft: '4px' }}>
              ({formatBytes(events.filter(e => e.type === 'progress').reduce((acc, e) => acc + (e as any).bytes, 0))})
            </span>
          )}
        </div>
      </div>

      <div className="progress-bar-container">
        <div 
          className={`progress-bar-fill ${percentage === undefined && !isComplete && !hasError ? 'indeterminate' : ''}`} 
          style={{ width: `${percentage !== undefined ? percentage : 100}%` }}
        />
      </div>

      <div className="logs-container">
        {events.map((evt, idx) => {
          if (evt.type === 'progress') {
            return (
              <div key={idx} className="log-line info">
                Extracting: {evt.file} <span style={{ opacity: 0.6 }}>({formatBytes(evt.bytes)})</span>
              </div>
            );
          }
          if (evt.type === 'warning') {
            return <div key={idx} className="log-line warn">WARN [{evt.code}]: {evt.file} - {evt.details}</div>;
          }
          if (evt.type === 'error') {
            return <div key={idx} className="log-line error">ERR  [{evt.code}]: {evt.details}</div>;
          }
          if (evt.type === 'complete') {
            return (
              <div key={idx} className="log-line success">
                Process finished. {evt.files_extracted} files verified, {evt.files_blocked} threats blocked.
              </div>
            );
          }
          return null;
        })}
        <div ref={logEndRef} />
      </div>

      <div className="panel-footer">
        {(isComplete || hasError) && (
          <button className="secondary-btn" onClick={onReset}>
            Dismiss
          </button>
        )}
        {isComplete && !hasError && (
          <button className="primary-btn" onClick={handleSaveTo}>
            Save to disk
          </button>
        )}
      </div>
    </div>
  );
};
