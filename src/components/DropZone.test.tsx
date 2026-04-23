import { render, screen, fireEvent, act } from '@testing-library/react';
import '@testing-library/jest-dom';
import { DropZone } from './DropZone';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

import React, { useState } from 'react';

const TestDropZone = (props: any) => {
  const [pwd, setPwd] = useState("");
  return <DropZone {...props} password={pwd} setPassword={setPwd} />;
};

describe('DropZone Component', () => {
  const mockOnAnalyzeStarted = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders correctly', () => {
    render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    expect(screen.getByText('Select Archive to Extract')).toBeInTheDocument();
  });

  it('handles drag over and drag leave correctly', () => {
    const { container } = render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    
    const dropZone = container.firstChild as HTMLElement;
    
    fireEvent.dragOver(dropZone);
    expect(dropZone.className).toContain('active');
    
    fireEvent.dragLeave(dropZone);
    expect(dropZone.className).not.toContain('active');

    fireEvent.drop(dropZone);
    expect(dropZone.className).not.toContain('active');
  });

  it('handles successful file selection', async () => {
    (open as jest.Mock).mockResolvedValue('/fake/path/test.zip');
    (invoke as jest.Mock).mockResolvedValue(true);
    
    const { container } = render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    const dropZone = container.firstChild as HTMLElement;
    fireEvent.click(dropZone);
    
    await act(async () => {
      await new Promise(process.nextTick);
    });
    
    expect(open).toHaveBeenCalled();
    expect(mockOnAnalyzeStarted).toHaveBeenCalledWith('/fake/path/test.zip', undefined);
    expect(invoke).toHaveBeenCalledWith('analyze_archive', { archivePath: '/fake/path/test.zip', password: undefined });
  });

  it('handles successful file selection with password', async () => {
    (open as jest.Mock).mockResolvedValue('/fake/path/test.zip');
    (invoke as jest.Mock).mockResolvedValue(true);
    
    const { container } = render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    
    // Type password
    const pwdInput = screen.getByPlaceholderText('Archive password (optional)');
    fireEvent.change(pwdInput, { target: { value: 'secret123' } });

    // Ensure clicking password input stop propagation doesn't trigger open dialog
    fireEvent.click(pwdInput);
    expect(open).not.toHaveBeenCalled();

    // Trigger select
    const dropZone = container.firstChild as HTMLElement;
    fireEvent.click(dropZone);
    
    await act(async () => {
      await new Promise(process.nextTick);
    });
    
    expect(open).toHaveBeenCalled();
    expect(mockOnAnalyzeStarted).toHaveBeenCalledWith('/fake/path/test.zip', 'secret123');
    expect(invoke).toHaveBeenCalledWith('analyze_archive', { archivePath: '/fake/path/test.zip', password: 'secret123' });
  });

  it('handles invalid file extension', async () => {
    (open as jest.Mock).mockResolvedValue('/fake/path/test.txt');
    const { container } = render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    const dropZone = container.firstChild as HTMLElement;
    fireEvent.click(dropZone);
    
    await act(async () => {
      await new Promise(process.nextTick);
    });
    
    expect(screen.getByText('Unsupported file type. Please use .zip, .rar, .tar, or .tgz')).toBeInTheDocument();
    expect(mockOnAnalyzeStarted).not.toHaveBeenCalled();
  });

  it('handles analysis error and manual dialog cancellation', async () => {
    console.error = jest.fn();
    (open as jest.Mock).mockResolvedValue('/fake/path/test.zip');
    (invoke as jest.Mock).mockRejectedValue('Backend error');
    
    const { container } = render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    const dropZone = container.firstChild as HTMLElement;
    fireEvent.click(dropZone);
    
    await act(async () => {
      await new Promise(process.nextTick);
    });
    
    expect(screen.getByText('Backend error')).toBeInTheDocument();
  });
  
  it('handles dialog rejection', async () => {
    console.error = jest.fn();
    (open as jest.Mock).mockRejectedValue('Dialog canceled');
    const { container } = render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    const dropZone = container.firstChild as HTMLElement;
    fireEvent.click(dropZone);
    await act(async () => {
      await new Promise(process.nextTick);
    });
    expect(screen.getByText('Dialog canceled')).toBeInTheDocument();
  });

  it('handles dialog resolving to null (user cancelled)', async () => {
    (open as jest.Mock).mockResolvedValue(null);
    const { container } = render(<TestDropZone onAnalyzeStarted={mockOnAnalyzeStarted} />);
    const dropZone = container.firstChild as HTMLElement;
    fireEvent.click(dropZone);
    await act(async () => {
      await new Promise(process.nextTick);
    });
    expect(mockOnAnalyzeStarted).not.toHaveBeenCalled();
  });
});
