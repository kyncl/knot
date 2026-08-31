// 3. Active Link Highlighting
function initActiveLinks() {
    const currentPathname = window.location.pathname.toLowerCase();
    const currentSegments = currentPathname.split('/').filter(Boolean);

    document.querySelectorAll('.sidebar-link, .sidebar-title-link').forEach(link => {
        const href = link.getAttribute('href');
        if (!href) return;

        // Use a temporary anchor to safely resolve relative paths (like ../)
        const parser = document.createElement('a');
        parser.href = href;

        const linkPathname = parser.pathname.toLowerCase();
        const linkSegments = linkPathname.split('/').filter(Boolean);

        if (currentSegments.length > 0 && linkSegments.length > 0) {
            // Compare the last 2 segments (e.g., folder name + filename) to distinguish
            // between nested/test.html and nested-two/test.html uniquely
            const currentTail = currentSegments.slice(-2).join('/');
            const linkTail = linkSegments.slice(-2).join('/');

            // Full path fallback for root-level single files (e.g., index.html)
            const currentFull = currentSegments.join('/');
            const linkFull = linkSegments.join('/');

            if (currentFull === linkFull || (currentTail === linkTail && currentTail !== '')) {
                link.classList.add('active');
            }
        }

        link.addEventListener('click', (e) => {
            // Prevent default ONLY if it's active and not an internal anchor jump
            if (link.classList.contains('active') && !link.hash) {
                e.preventDefault();
            }
        });
    });
}
