import { useState, useEffect, useRef } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from '@tauri-apps/api/core';
import { DropZone } from "./components/DropZone";
import { ProgressPanel, SandboxEvent } from "./components/ProgressPanel";
import { AboutModal } from "./components/AboutModal";

function App() {
  const [isProcessing, setIsProcessing] = useState(false);
  const [events, setEvents] = useState<SandboxEvent[]>([]);
  const [isComplete, setIsComplete] = useState(false);
  const [hasError, setHasError] = useState(false);
  const [isAboutOpen, setIsAboutOpen] = useState(false);
  const [password, setPassword] = useState("");
  const passwordRef = useRef("");

  // Sync ref with state
  useEffect(() => {
    passwordRef.current = password;
  }, [password]);

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

    listen<{ paths: string[] }>("tauri://drag-drop", async (event) => {
      if (!isProcessing && event.payload.paths.length > 0) {
        const path = event.payload.paths[0].toLowerCase();
        
        if (!path.endsWith('.zip') && !path.endsWith('.tar') && !path.endsWith('.tar.gz') && !path.endsWith('.tgz') && !path.endsWith('.rar')) {
          setHasError(true);
          setEvents([{ type: 'error', code: 'UNSUPPORTED', details: 'Unsupported file type. Please use .zip, .rar, .tar, or .tgz', file: path, current: 0, total: 0, bytes: 0 }]);
          setIsComplete(true);
          return;
        }

        handleAnalyzeStarted(path);
        try {
          const pwdArg = passwordRef.current.trim() ? passwordRef.current.trim() : undefined;
          await invoke('analyze_archive', { archivePath: event.payload.paths[0], password: pwdArg });
        } catch (e) {
          setHasError(true);
          setEvents([{ type: 'error', code: 'RUNTIME_ERROR', details: String(e), file: path, current: 0, total: 0, bytes: 0 }]);
        }
      }
    }).then(f => {
      if (!isMounted) f(); else unlistenDrop = f;
    });

    let unlistenAbout: UnlistenFn | undefined;
    listen("open-about", () => {
      setIsAboutOpen(true);
    }).then(f => {
      if (!isMounted) f(); else unlistenAbout = f;
    });

    return () => {
      isMounted = false;
      if (unlisten) unlisten();
      if (unlistenDrop) unlistenDrop();
      if (unlistenAbout) unlistenAbout();
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
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ color: 'var(--brand)' }}>
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
            <line x1="12" y1="22.08" x2="12" y2="12"></line>
          </svg>
          <span className="text-gradient">ZDefuser</span>
        </h1>
        {!isProcessing && <p>Zero-Trust Sandboxed Extraction</p>}
      </div>

      <div className="main-content">
        {!isProcessing ? (
          <DropZone 
            onAnalyzeStarted={handleAnalyzeStarted} 
            password={password}
            setPassword={setPassword}
          />
        ) : (
          <ProgressPanel
            events={events}
            isComplete={isComplete}
            hasError={hasError}
            onReset={handleReset}
          />
        )}
      </div>

      <AboutModal isOpen={isAboutOpen} onClose={() => setIsAboutOpen(false)} />
    </div>
  );
}

export default App;
