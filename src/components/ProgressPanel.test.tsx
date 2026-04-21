import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ProgressPanel } from './ProgressPanel';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

describe('ProgressPanel Component', () => {
  const mockOnReset = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    (HTMLElement.prototype as any).scrollIntoView = jest.fn();
  });

  it('renders initial empty state gracefully', () => {
    render(<ProgressPanel events={[]} isComplete={false} hasError={false} onReset={mockOnReset} />);
    expect(screen.getByText('Sandbox Execution')).toBeInTheDocument();
    expect(screen.getByText('Processing')).toBeInTheDocument();
  });

  it('renders progress and warning events properly', () => {
    const events: any = [
      { type: 'progress', file: 'A.txt', total: 100, current: 50 },
      { type: 'warning', code: 'PATH_TRAVERSAL', file: 'B.sh', details: 'Alert' }
    ];
    render(<ProgressPanel events={events} isComplete={false} hasError={false} onReset={mockOnReset} />);
    
    expect(screen.getByText(/Extracting: A.txt/)).toBeInTheDocument();
    expect(screen.getByText(/WARN \[PATH_TRAVERSAL\]: B.sh - Alert/)).toBeInTheDocument();
  });

  it('displays completion correctly when verified', () => {
    const events: any = [
      { type: 'complete', files_extracted: 10, files_blocked: 0, total_bytes: 100 }
    ];
    render(<ProgressPanel events={events} isComplete={true} hasError={false} onReset={mockOnReset} />);
    
    expect(screen.getByText('Verified')).toBeInTheDocument();
    expect(screen.getByText(/Process finished. 10 files verified, 0 threats blocked./)).toBeInTheDocument();
    
    expect(screen.getByText('Dismiss')).toBeInTheDocument();
    expect(screen.getByText('Save to disk')).toBeInTheDocument();
  });

  it('displays error badge when hasError is true and handles unknown event types', () => {
    const events: any = [
      { type: 'error', code: 'ZIP_BOMB', details: 'Memory Limit' },
      { type: 'unknown_fake_type' } // Triggers the return null fallback
    ];
    render(<ProgressPanel events={events} isComplete={true} hasError={true} onReset={mockOnReset} />);
    expect(screen.getByText('Blocked')).toBeInTheDocument();
    expect(screen.getByText(/ERR \[ZIP_BOMB\]: Memory Limit/)).toBeInTheDocument();
  });

  it('calls onReset when dismiss button is clicked', () => {
    render(<ProgressPanel events={[]} isComplete={true} hasError={false} onReset={mockOnReset} />);
    fireEvent.click(screen.getByText('Dismiss'));
    expect(mockOnReset).toHaveBeenCalled();
  });

  it('calls save process correctly', async () => {
    (open as jest.Mock).mockResolvedValue('/foo/bar');
    (invoke as jest.Mock).mockResolvedValue(true);
    // Suppress alert during test
    window.alert = jest.fn();
    
    render(<ProgressPanel events={[]} isComplete={true} hasError={false} onReset={mockOnReset} />);
    fireEvent.click(screen.getByText('Save to disk'));
    
    await new Promise(process.nextTick);
    
    expect(open).toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith('release_files', { targetDir: '/foo/bar' });
    expect(window.alert).toHaveBeenCalledWith("Extraction Completed and Saved!");
  });

  it('handles save process rejection', async () => {
    (open as jest.Mock).mockRejectedValue('Canceled by user');
    window.alert = jest.fn();
    console.error = jest.fn();
    render(<ProgressPanel events={[]} isComplete={true} hasError={false} onReset={mockOnReset} />);
    fireEvent.click(screen.getByText('Save to disk'));
    await new Promise(process.nextTick);
    expect(window.alert).toHaveBeenCalledWith("Failed to save: Canceled by user");
  });

  it('handles save process resolving to null (user cancelled dialog)', async () => {
    (open as jest.Mock).mockResolvedValue(null);
    render(<ProgressPanel events={[]} isComplete={true} hasError={false} onReset={mockOnReset} />);
    fireEvent.click(screen.getByText('Save to disk'));
    await new Promise(process.nextTick);
    expect(invoke).not.toHaveBeenCalled();
  });
});
