const initSidebarState = () => {
    const normalizePath = (path) => path.replace(/\/index\.html$/, '/').replace(/\/$/, '');
    const currentPath = normalizePath(window.location.pathname);

    document.querySelectorAll('.sidebar-section').forEach((section) => {
        const links = section.querySelectorAll('a.sidebar-title-link, a.sidebar-link');
        let isSectionActive = false;

        links.forEach((link) => {
            if (link.classList.contains('active')) {
                isSectionActive = true;
            } else {
                try {
                    const linkPath = normalizePath(new URL(link.getAttribute('href'), window.location.href).pathname);
                    if (linkPath === currentPath) {
                        isSectionActive = true;
                        link.classList.add('active');
                    }
                } catch (e) {
                    // Ignore invalid URLs
                }
            }
        });
        if (isSectionActive) {
            section.classList.remove('collapsed');
        } else {
            section.classList.add('collapsed');
        }
    });
};

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initSidebarState);
} else {
    initSidebarState();
}

document.addEventListener('click', (e) => {
    const toggleBtn = e.target.closest('.sidebar-toggle');
    if (!toggleBtn) return;

    const section = toggleBtn.closest('.sidebar-section');
    if (section) {
        section.classList.toggle('collapsed');
    }
});
