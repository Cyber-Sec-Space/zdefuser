import { useState, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from '@tauri-apps/api/core';
import { DropZone } from "./components/DropZone";
import { ProgressPanel, SandboxEvent } from "./components/ProgressPanel";

function App() {
  const [isProcessing, setIsProcessing] = useState(false);
  const [events, setEvents] = useState<SandboxEvent[]>([]);
  const [isComplete, setIsComplete] = useState(false);
  const [hasError, setHasError] = useState(false);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let unlistenDrop: UnlistenFn | undefined;
    let isMounted = true;

    listen<SandboxEvent>("sandbox_event", (event) => {
      const data = event.payload;
      setEvents((prev) => [...prev, data]);
      
      if (data.type === 'complete') {
        setIsComplete(true);
      } else if (data.type === 'error' || data.type === 'warning' && data.code.includes('BLOCKED')) {
        if (data.type === 'error') {
          setHasError(true);
          setIsComplete(true); 
        }
      }
    }).then(f => {
      if (!isMounted) f(); else unlisten = f;
    });

    listen<{paths: string[]}>("tauri://drag-drop", async (event) => {
      if (!isProcessing && event.payload.paths.length > 0) {
         const path = event.payload.paths[0];
         handleAnalyzeStarted(path);
         try {
           // We don't have a password prompt for global drag-drop yet so it's undefined
           await invoke('analyze_archive', { archivePath: path });
         } catch(e) {
           setHasError(true);
         }
      }
    }).then(f => {
      if (!isMounted) f(); else unlistenDrop = f;
    });

    return () => {
      isMounted = false;
      if (unlisten) unlisten();
      if (unlistenDrop) unlistenDrop();
    };
  }, [isProcessing]);

  const handleAnalyzeStarted = (_path: string, _password?: string) => {
    setIsProcessing(true);
    setEvents([]);
    setIsComplete(false);
    setHasError(false);
  };

  const handleReset = () => {
    setIsProcessing(false);
    setEvents([]);
    setIsComplete(false);
    setHasError(false);
  };

  return (
    <div className="app-container">
      <div className="header">
        <h1>
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{color: 'var(--brand)'}}>
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
            <line x1="12" y1="22.08" x2="12" y2="12"></line>
          </svg>
          Z<span className="text-gradient">Defuser</span>
        </h1>
        <p>Zero-Trust Sandboxed Extraction</p>
      </div>

      <div className="main-content">
         {!isProcessing ? (
          <DropZone onAnalyzeStarted={handleAnalyzeStarted} />
        ) : (
          <ProgressPanel 
             events={events} 
             isComplete={isComplete} 
             hasError={hasError} 
             onReset={handleReset} 
          />
        )}
      </div>
    </div>
  );
}

export default App;
