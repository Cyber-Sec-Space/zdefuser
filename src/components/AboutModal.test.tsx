import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { AboutModal } from './AboutModal';

describe('AboutModal', () => {
  it('does not render when isOpen is false', () => {
    const { container } = render(<AboutModal isOpen={false} onClose={jest.fn()} />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders correctly when isOpen is true', () => {
    render(<AboutModal isOpen={true} onClose={jest.fn()} />);
    expect(screen.getByText('ZDefuser - About & Licenses')).toBeInTheDocument();
    expect(screen.getByText('Version 1.0.0-rc.1')).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', () => {
    const onClose = jest.fn();
    render(<AboutModal isOpen={true} onClose={onClose} />);
    const closeBtn = screen.getByRole('button');
    fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when overlay is clicked', () => {
    const onClose = jest.fn();
    const { container } = render(<AboutModal isOpen={true} onClose={onClose} />);
    // The first div is the overlay
    const overlay = container.firstChild as Element;
    fireEvent.click(overlay);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('does not call onClose when modal content is clicked (propagation stopped)', () => {
    const onClose = jest.fn();
    render(<AboutModal isOpen={true} onClose={onClose} />);
    const content = screen.getByText('ZDefuser - About & Licenses').parentElement?.parentElement;
    fireEvent.click(content!);
    expect(onClose).not.toHaveBeenCalled();
  });

  it('calls onClose on pressing Escape key', () => {
    const onClose = jest.fn();
    render(<AboutModal isOpen={true} onClose={onClose} />);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('does not call onClose on pressing other keys', () => {
    const onClose = jest.fn();
    render(<AboutModal isOpen={true} onClose={onClose} />);
    fireEvent.keyDown(window, { key: 'Enter' });
    expect(onClose).not.toHaveBeenCalled();
  });
});
