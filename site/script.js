(function() {
    'use strict';

    // Initialize on DOM load
    document.addEventListener('DOMContentLoaded', function() {
        initGlitchEffect();
        initHoverEffects();
        initParallax();
        initCRTEffect();
        initReleaseDownloads();
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
        if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            return;
        }
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

    // Resolve GitHub release assets: links when present; optional hide for Portable (.exe); else muted label.
    function initReleaseDownloads() {
        const owner = 'Aspenini';
        const repo = 'Memory-Pak';
        const latestReleaseUrl = `https://api.github.com/repos/${owner}/${repo}/releases/latest`;
        const releaseVersionEl = document.getElementById('release-version');
        const downloadsSection = document.querySelector('.downloads');
        const ERROR_SLOT =
            'Could not verify this release — try the GitHub releases page.';

        if (downloadsSection) {
            downloadsSection.classList.add('release-loading');
        }

        function updateJsonLdVersion(tagName) {
            if (!tagName) return;
            const ldScript = document.querySelector('script[type="application/ld+json"]');
            if (!ldScript) return;
            try {
                const data = JSON.parse(ldScript.textContent);
                data.softwareVersion = String(tagName).replace(/^v/i, '');
                ldScript.textContent = JSON.stringify(data);
            } catch (_e) {
                /* ignore */
            }
        }

        function markUnavailable(target, overrideMessage) {
            const message =
                overrideMessage ||
                target.textContent.trim() ||
                target.getAttribute('data-release-empty-label') ||
                'Download';
            const span = document.createElement('span');
            span.className = 'download-badge download-badge--missing';
            span.textContent = message;
            if (target.parentNode) {
                target.parentNode.replaceChild(span, target);
            }
        }

        function markAllReleaseSlotsUnavailable(message) {
            document.querySelectorAll('a.download-badge[data-release-match]').forEach((el) => {
                if (el.hasAttribute('data-release-hide-if-missing')) {
                    el.remove();
                    return;
                }
                markUnavailable(el, message);
            });
        }

        function finishReleaseState() {
            if (downloadsSection) {
                downloadsSection.classList.remove('release-loading');
            }
        }

        fetch(latestReleaseUrl, {
            headers: {
                'Accept': 'application/vnd.github+json'
            }
        })
            .then(response => {
                if (!response.ok) {
                    throw new Error(`GitHub API request failed: ${response.status}`);
                }
                return response.json();
            })
            .then(release => {
                if (!release || !Array.isArray(release.assets) || release.assets.length === 0) {
                    if (releaseVersionEl) {
                        releaseVersionEl.textContent =
                            'Latest release: no assets attached yet — see GitHub for files.';
                    }
                    markAllReleaseSlotsUnavailable(null);
                    finishReleaseState();
                    return;
                }

                const assets = release.assets;
                const tag = release.tag_name || 'latest';
                if (releaseVersionEl) {
                    releaseVersionEl.textContent = `Latest release: ${tag}`;
                }
                updateJsonLdVersion(tag);

                const targets = Array.from(document.querySelectorAll('a.download-badge[data-release-match]'));

                targets.forEach(target => {
                    const rawMatch = target.getAttribute('data-release-match');
                    if (!rawMatch) return;

                    const keywords = rawMatch
                        .split(',')
                        .map(part => part.trim().toLowerCase())
                        .filter(Boolean);
                    if (keywords.length === 0) return;

                    const matchedAsset = assets.find(asset => {
                        const name = (asset && asset.name ? asset.name : '').toLowerCase();
                        return keywords.every(keyword => name.includes(keyword));
                    });

                    if (!matchedAsset || !matchedAsset.browser_download_url) {
                        if (target.hasAttribute('data-release-hide-if-missing')) {
                            target.remove();
                            return;
                        }
                        markUnavailable(target);
                        return;
                    }

                    if (target.tagName === 'A') {
                        target.href = matchedAsset.browser_download_url;
                        target.setAttribute('download', matchedAsset.name || '');
                        target.title = `From release ${tag}`;
                    }
                });

                finishReleaseState();
            })
            .catch(error => {
                if (releaseVersionEl) {
                    releaseVersionEl.textContent =
                        'Latest release: could not load from GitHub. Use the releases page to download.';
                }
                markAllReleaseSlotsUnavailable(ERROR_SLOT);
                finishReleaseState();
                console.warn('Unable to load latest release assets.', error);
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

