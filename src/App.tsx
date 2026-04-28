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
        
        if (!path.endsWith('.zip') && !path.endsWith('.tar') && !path.endsWith('.tar.gz') && !path.endsWith('.tgz') && !path.endsWith('.rar') && !path.endsWith('.7z')) {
          handleAnalyzeStarted(path);
          setHasError(true);
          setEvents([{ type: 'error', code: 'UNSUPPORTED', details: 'Unsupported file type. Please use .zip, .rar, .7z, .tar, or .tgz' }]);
          setIsComplete(true);
          return;
        }

        handleAnalyzeStarted(path);
        try {
          const pwdArg = passwordRef.current.trim() ? passwordRef.current.trim() : undefined;
          await invoke('analyze_archive', { archivePath: event.payload.paths[0], password: pwdArg });
        } catch (e) {
          setHasError(true);
          setEvents([{ type: 'error', code: 'RUNTIME_ERROR', details: String(e) }]);
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
          <img src="/logo.png" alt="ZDefuser Logo" width={28} height={28} style={{ backgroundColor: 'transparent' }} />
          <span className="text-gradient">Defuser</span>
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
