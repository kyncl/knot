// 2. Sidebar Toggle & State Management
function initSidebar() {
    const toggleBtn = document.getElementById('sidebarToggle');
    const backdrop = document.getElementById('sidebarBackdrop');
    const sidebar = document.querySelector('.sidebar');
    const sidebarLinks = document.querySelectorAll('.sidebar-link');

    const setSidebarOpen = (isOpen) => {
        if (sidebar) sidebar.classList.toggle('active', isOpen);
        if (backdrop) backdrop.classList.toggle('active', isOpen);
        if (toggleBtn) {
            toggleBtn.classList.toggle('open', isOpen);
            toggleBtn.setAttribute('aria-expanded', isOpen);
        }
        // Toggle both so CSS visibility and search locking both work seamlessly
        document.body.classList.toggle('sidebar-open', isOpen);
        document.body.classList.toggle('sidebar-locked', isOpen);
    };

    if (toggleBtn) {
        toggleBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            const isOpen = !document.body.classList.contains('sidebar-open');
            setSidebarOpen(isOpen);
        });
    }

    if (backdrop) {
        backdrop.addEventListener('click', () => setSidebarOpen(false));
    }

    sidebarLinks.forEach(link => {
        link.addEventListener('click', () => setSidebarOpen(false));
    });
}
