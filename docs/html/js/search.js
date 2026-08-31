// 4. Search Functionality & Indexing
function initSearch() {
    const searchBtn = document.querySelector('.search-btn');
    const searchBox = document.querySelector('.search-box');
    const searchInput = document.getElementById('search');
    const suggestionsContainer = document.querySelector('.search-suggestions');

    if (!searchBtn || !searchBox || !searchInput || !suggestionsContainer) return;

    let currentMatches = [];
    let selectedIndex = 0;
    const searchIndex = [];

    const pageIcon = `<svg width="16" height="16" viewBox="0 0 20 20" style="flex-shrink: 0;">
<path d="M17 6v12c0 .52-.2 1-1 1H4c-.7 0-1-.33-1-1V2c0-.55.42-1 1-1h8l5 5zM14 8h-3.13c-.51 0-.87-.34-.87-.87V4" stroke="currentColor" fill="none" fill-rule="evenodd" stroke-linejoin="round"></path>
</svg>`.replaceAll("\n", " ");
    const headerIcon = `<svg stroke="currentColor" fill="currentColor" stroke-width="0" viewBox="0 0 24 24" height="16" width="16" xmlns="http://www.w3.org/2000/svg">
<g id="Hashtag">
<path d="M20.435,15.506H16.2l.61-7h3.63a.5.5,0,0,0,.5-.5.5.5,0,0,0-.5-.5H16.9l.34-3.87a.509.509,0,0,0-.46-.54.5.5,0,0,0-.54.46l-.35,3.95H8.9l.34-3.87a.509.509,0,0,0-.46-.54.491.491,0,0,0-.54.46l-.35,3.95H3.565a.5.5,0,0,0-.5.5.5.5,0,0,0,.5.5h4.24l-.62,7H3.565a.5.5,0,0,0-.5.5.5.5,0,0,0,.5.5h3.54l-.34,3.86a.508.508,0,0,0,.45.54h.05a.516.516,0,0,0,.5-.46l.34-3.94h7l-.34,3.86a.508.508,0,0,0,.45.54h.05a.516.516,0,0,0,.5-.46l.34-3.94h4.33a.5.5,0,0,0,.5-.5A.5.5,0,0,0,20.435,15.506Zm-5.25,0H8.2l.61-7h7Z"></path>
</g>
</svg>`.replaceAll("\n", " ");

    // Normalize pathnames to reliably detect the active page
    const normalizePath = (path) => path.replace(/\/index\.html$/, '/').replace(/\/$/, '');
    const currentPath = normalizePath(window.location.pathname);

    const isCurrentPage = (href) => {
        try {
            const targetPath = normalizePath(new URL(href, window.location.href).pathname);
            return targetPath === currentPath;
        } catch (e) {
            return false;
        }
    };

    // Calculate relative depth prefix (e.g., "../../") to navigate from current subfolder to root
    const sampleLink = document.querySelector('.sidebar-link, .sidebar-title-link');
    const sampleHref = sampleLink ? sampleLink.getAttribute('href') : '';
    const prefixMatch = sampleHref ? sampleHref.match(/^(?:\.\.\/)+/) : null;
    const prefix = prefixMatch ? prefixMatch[0] : '';

    // Ingest global Vault search index injected via Rust script
    if (Array.isArray(window.VAULT_SEARCH_INDEX)) {
        window.VAULT_SEARCH_INDEX.forEach(item => {
            const fullUrl = `${prefix}${item.url}`;
            const category = item.category || 'Page';

            // Resolve page title: root index -> "Homepage", nested index -> parent folder name
            let title = item.title;
            const isIndexPage = item.title.toLowerCase() === 'index' ||
                item.url === 'index.html' ||
                item.url.endsWith('/index.html');

            if (isIndexPage) {
                title = item.category ? item.category : 'Homepage';
            }

            // Add Page Entry (only if it is not the current page)
            if (!isCurrentPage(fullUrl)) {
                searchIndex.push({
                    title,
                    searchableText: `${title} ${category}`.toLowerCase(),
                    href: fullUrl,
                    type: category,
                    isHeader: false
                });
            }

            // Add Section/Heading Entries
            if (Array.isArray(item.headers)) {
                item.headers.forEach(header => {
                    // Lowercase, replace non-word chars with hyphens, and trim leading/trailing hyphens
                    const id = header
                        .toLowerCase()
                        .replace(/&amp;/g, 'amp')
                        .replace(/&/g, 'amp')
                        .replace(/[^\w]+/g, '-')
                        .replace(/^-+|-+$/g, '');
                    if (!id) return; // Skip if header consisted purely of special characters

                    searchIndex.push({
                        title: header,
                        searchableText: `${header} ${title} ${category}`.toLowerCase(),
                        href: `${fullUrl}#${id}`,
                        type: `${title} > Section`,
                        isHeader: true
                    });
                });
            }
        });
    } else {
        // Fallback DOM Indexing if VAULT_SEARCH_INDEX is unavailable
        document.querySelectorAll('.sidebar-link').forEach(link => {
            const href = link.getAttribute('href');
            if (href && !isCurrentPage(href)) {
                const title = link.textContent.trim();
                searchIndex.push({
                    title,
                    searchableText: title.toLowerCase(),
                    href,
                    type: 'Page',
                    isHeader: false
                });
            }
        });
    }

    const closeSearch = () => {
        searchBox.classList.remove('active');
        searchInput.value = '';
        suggestionsContainer.innerHTML = '';
        currentMatches = [];
        selectedIndex = 0;
    };

    const updateHighlight = () => {
        suggestionsContainer.querySelectorAll('.suggestion-item').forEach((btn, index) => {
            btn.classList.toggle('selected', index === selectedIndex);
        });
    };

    const updateSuggestions = (query) => {
        suggestionsContainer.innerHTML = '';
        const trimmedQuery = query.trim().toLowerCase();

        if (!trimmedQuery) {
            currentMatches = searchIndex.slice(0, 8);
        } else {
            currentMatches = searchIndex.map(item => {
                const titleLower = item.title.toLowerCase();
                let score = 0;

                if (titleLower === trimmedQuery) score = 100;
                else if (titleLower.startsWith(trimmedQuery)) score = 75;
                else if (titleLower.includes(trimmedQuery)) score = 50;
                else if (item.searchableText.includes(trimmedQuery)) score = 25;

                return { ...item, score };
            })
                .filter(item => item.score > 0)
                .sort((a, b) => b.score - a.score)
                .slice(0, 6);
        }

        if (currentMatches.length === 0) {
            suggestionsContainer.innerHTML = `<div style="padding: 0.6rem 0.75rem; color: var(--muted); font-size: 0.9rem; font-family: var(--font-code);">No results found</div>`;
            selectedIndex = -1;
            return;
        }

        selectedIndex = 0;

        currentMatches.forEach((match, index) => {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = `suggestion-item ${index === 0 ? 'selected' : ''}`;
            const icon = match.isHeader ? headerIcon : pageIcon;

            btn.innerHTML = `
                <span style="display: inline-flex; align-items: center; gap: 0.5rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                    ${icon}
                    <span>${match.title}</span>
                </span>
                <small style="opacity: 0.4; font-size: 0.75rem; font-family: var(--font-code); margin-left: 0.5rem; flex-shrink: 0;">${match.type}</small>
            `;

            btn.addEventListener('click', () => {
                window.location.href = match.href;
                closeSearch();
            });

            suggestionsContainer.appendChild(btn);
        });
    };

    searchInput.addEventListener('input', (e) => updateSuggestions(e.target.value));

    searchInput.addEventListener('keydown', (e) => {
        if (currentMatches.length === 0) return;

        if (e.key === 'ArrowDown') {
            e.preventDefault();
            selectedIndex = (selectedIndex + 1) % currentMatches.length;
            updateHighlight();
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            selectedIndex = (selectedIndex - 1 + currentMatches.length) % currentMatches.length;
            updateHighlight();
        } else if (e.key === 'Enter') {
            e.preventDefault();
            if (selectedIndex >= 0 && currentMatches[selectedIndex]) {
                window.location.href = currentMatches[selectedIndex].href;
                closeSearch();
            }
        }
    });

    document.addEventListener('keydown', (e) => {
        if (document.body.classList.contains('sidebar-locked')) return;

        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
            e.preventDefault();
            searchBox.classList.add('active');
            searchInput.focus();
            updateSuggestions(searchInput.value);
        } else if (e.key === '/' && document.activeElement.tagName !== 'INPUT' && document.activeElement.tagName !== 'TEXTAREA') {
            e.preventDefault();
            searchBox.classList.add('active');
            searchInput.focus();
            updateSuggestions(searchInput.value);
        } else if (e.key === 'Escape') {
            closeSearch();
        }
    });

    searchBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        if (document.body.classList.contains('sidebar-locked')) return;

        if (searchBox.classList.contains('active')) {
            closeSearch();
        } else {
            searchBox.classList.add('active');
            searchInput.focus();
            updateSuggestions(searchInput.value);
        }
    });

    searchBox.addEventListener('click', (e) => e.stopPropagation());
    document.addEventListener('click', closeSearch);
}
