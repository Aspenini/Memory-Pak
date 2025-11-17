(function() {
    'use strict';

    // Initialize on DOM load
    document.addEventListener('DOMContentLoaded', function() {
        initGlitchEffect();
        initHoverEffects();
        initParallax();
        initCRTEffect();
        initCargoCard();
    });

    // Enhanced glitch effect
    function initGlitchEffect() {
        const glitchEl = document.querySelector('.glitch');
        if (!glitchEl) return;

        let glitchInterval;
        
        function triggerGlitch() {
            glitchEl.style.animation = 'none';
            setTimeout(() => {
                glitchEl.style.animation = 'glitch-text 0.3s ease-in-out';
            }, 10);
        }

        // Random glitch trigger
        setInterval(() => {
            if (Math.random() > 0.7) {
                triggerGlitch();
            }
        }, 3000);

        // Mouse hover glitch
        glitchEl.addEventListener('mouseenter', function() {
            clearInterval(glitchInterval);
            glitchInterval = setInterval(triggerGlitch, 200);
        });

        glitchEl.addEventListener('mouseleave', function() {
            clearInterval(glitchInterval);
        });
    }

    // Download card hover effects
    function initHoverEffects() {
        const downloadCards = document.querySelectorAll('.download-card');
        
        downloadCards.forEach(card => {
            // Only prevent default for placeholder links (href="#")
            if (card.tagName === 'A' && card.getAttribute('href') === '#') {
                card.addEventListener('click', function(e) {
                    e.preventDefault();
                    // Add click flash effect
                    this.style.transition = 'none';
                    this.style.filter = 'brightness(2)';
                    setTimeout(() => {
                        this.style.transition = '';
                        this.style.filter = '';
                    }, 150);
                });
            } else if (card.tagName === 'A' && card.getAttribute('href') !== '#') {
                // For working download links, add flash effect on click but allow download
                card.addEventListener('click', function(e) {
                    // Add click flash effect
                    this.style.transition = 'none';
                    this.style.filter = 'brightness(2)';
                    setTimeout(() => {
                        this.style.transition = '';
                        this.style.filter = '';
                    }, 150);
                    // Let the browser handle the download
                });
            }
            
            // Handle Cargo card - add hover effects
            if (card.classList.contains('cargo-card')) {
                card.addEventListener('mouseenter', function() {
                    this.style.transform = 'scale(1.05)';
                });
                card.addEventListener('mouseleave', function() {
                    this.style.transform = '';
                });
            }
        });
    }

    // Subtle parallax effect
    function initParallax() {
        let ticking = false;
        
        function updateParallax() {
            const scrolled = window.pageYOffset;
            const hero = document.querySelector('.hero');
            const features = document.querySelector('.features');
            
            if (hero) {
                hero.style.transform = `translateY(${scrolled * 0.2}px)`;
            }
            
            if (features) {
                features.style.transform = `translateY(${scrolled * 0.1}px)`;
            }
            
            ticking = false;
        }

        window.addEventListener('scroll', function() {
            if (!ticking) {
                window.requestAnimationFrame(updateParallax);
                ticking = true;
            }
        });
    }

    // CRT screen effect variations
    function initCRTEffect() {
        const crtScreen = document.querySelector('.crt-screen');
        if (!crtScreen) return;

        // Random brightness flicker
        setInterval(() => {
            if (Math.random() > 0.95) {
                crtScreen.style.opacity = '0.98';
                setTimeout(() => {
                    crtScreen.style.opacity = '1';
                }, 50);
            }
        }, 500);

        // Screen curvature effect on resize
        function updateScreenCurvature() {
            const width = window.innerWidth;
            if (width > 768) {
                crtScreen.style.borderRadius = '10px';
                crtScreen.style.boxShadow = '0 0 100px rgba(0, 255, 0, 0.3)';
            } else {
                crtScreen.style.borderRadius = '0';
                crtScreen.style.boxShadow = 'none';
            }
        }

        window.addEventListener('resize', updateScreenCurvature);
        updateScreenCurvature();
    }

    // Add noise effect overlay
    function addNoiseOverlay() {
        const canvas = document.createElement('canvas');
        canvas.style.position = 'fixed';
        canvas.style.top = '0';
        canvas.style.left = '0';
        canvas.style.width = '100%';
        canvas.style.height = '100%';
        canvas.style.pointerEvents = 'none';
        canvas.style.opacity = '0.03';
        canvas.style.zIndex = '1002';
        canvas.style.mixBlendMode = 'screen';
        
        const ctx = canvas.getContext('2d');
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        
        function drawNoise() {
            const imageData = ctx.createImageData(canvas.width, canvas.height);
            const data = imageData.data;
            
            for (let i = 0; i < data.length; i += 4) {
                const noise = Math.random() * 255;
                data[i] = noise;
                data[i + 1] = noise;
                data[i + 2] = noise;
                data[i + 3] = 255;
            }
            
            ctx.putImageData(imageData, 0, 0);
        }
        
        function animateNoise() {
            drawNoise();
            requestAnimationFrame(animateNoise);
        }
        
        document.body.appendChild(canvas);
        animateNoise();
        
        window.addEventListener('resize', () => {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
        });
    }

    // Initialize noise after a short delay
    setTimeout(addNoiseOverlay, 500);

    // Keyboard navigation enhancement
    document.addEventListener('keydown', function(e) {
        const downloadCards = Array.from(document.querySelectorAll('.download-card'));
        
        if (e.key === 'Tab' && downloadCards.length > 0) {
            downloadCards.forEach(card => {
                card.addEventListener('focus', function() {
                    this.style.outline = '3px solid var(--text-accent)';
                    this.style.outlineOffset = '3px';
                });
                
                card.addEventListener('blur', function() {
                    this.style.outline = 'none';
                });
            });
        }
    });

    // Cargo card copy functionality
    function initCargoCard() {
        const copyBtn = document.getElementById('cargo-copy-btn');
        if (!copyBtn) return;
        
        copyBtn.addEventListener('click', function(e) {
            e.preventDefault();
            e.stopPropagation();
            
            const command = 'cargo install Memory-Pak';
            navigator.clipboard.writeText(command).then(() => {
                const copyText = this.querySelector('.copy-text');
                const originalText = copyText.textContent;
                copyText.textContent = 'Copied!';
                setTimeout(() => {
                    copyText.textContent = originalText;
                }, 2000);
            }).catch(err => {
                console.error('Failed to copy:', err);
                // Fallback for older browsers
                const textArea = document.createElement('textarea');
                textArea.value = command;
                textArea.style.position = 'fixed';
                textArea.style.opacity = '0';
                document.body.appendChild(textArea);
                textArea.select();
                try {
                    document.execCommand('copy');
                    const copyText = this.querySelector('.copy-text');
                    const originalText = copyText.textContent;
                    copyText.textContent = 'Copied!';
                    setTimeout(() => {
                        copyText.textContent = originalText;
                    }, 2000);
                } catch (err) {
                    console.error('Fallback copy failed:', err);
                }
                document.body.removeChild(textArea);
            });
        });
    }

    // Performance optimization: Reduce animations on slow devices
    if (navigator.hardwareConcurrency && navigator.hardwareConcurrency < 4) {
        document.body.classList.add('low-performance');
        const style = document.createElement('style');
        style.textContent = `
            .low-performance .glitch::before,
            .low-performance .glitch::after {
                display: none;
            }
            .low-performance .crt-screen::before {
                animation: none;
            }
        `;
        document.head.appendChild(style);
    }
})();

