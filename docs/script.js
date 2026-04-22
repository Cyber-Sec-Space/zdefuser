document.addEventListener('DOMContentLoaded', () => {
    // 1. Header Scroll Effect
    const header = document.querySelector('header');
    
    window.addEventListener('scroll', () => {
        if (window.scrollY > 50) {
            header.style.background = 'rgba(5, 5, 10, 0.9)';
            header.style.borderBottomColor = 'rgba(255, 255, 255, 0.15)';
        } else {
            header.style.background = 'rgba(5, 5, 10, 0.5)';
            header.style.borderBottomColor = 'rgba(255, 255, 255, 0.08)';
        }
    });

    // 2. Intersection Observer for Fade-In-Up Animations
    const observerOptions = {
        root: null,
        rootMargin: '0px',
        threshold: 0.15
    };

    const observer = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('visible');
                // Optional: Stop observing once it has animated
                // observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    // Get all elements with the fade-in-up class
    const fadeElements = document.querySelectorAll('.fade-in-up');
    fadeElements.forEach(el => observer.observe(el));
    
    // 3. Optional: Add interactive 3D tilt effect on mockups
    const mockups = document.querySelectorAll('.app-mockup, .concept-art');
    
    mockups.forEach(img => {
        img.addEventListener('mousemove', (e) => {
            const rect = img.getBoundingClientRect();
            const x = e.clientX - rect.left; // x position within the element.
            const y = e.clientY - rect.top;  // y position within the element.
            
            const centerX = rect.width / 2;
            const centerY = rect.height / 2;
            
            const rotateX = ((y - centerY) / centerY) * -5;
            const rotateY = ((x - centerX) / centerX) * 5;
            
            img.style.transform = `perspective(1000px) rotateX(${rotateX}deg) rotateY(${rotateY}deg) scale3d(1.02, 1.02, 1.02)`;
        });
        
        img.addEventListener('mouseleave', () => {
            img.style.transform = `perspective(1000px) rotateX(0deg) rotateY(0deg) scale3d(1, 1, 1)`;
        });
        
        img.style.transition = 'transform 0.1s ease-out';
        img.addEventListener('mouseleave', () => {
            img.style.transition = 'transform 0.5s ease-out';
        });
        img.addEventListener('mouseenter', () => {
            img.style.transition = 'transform 0.1s ease-out';
        });
    });
});
