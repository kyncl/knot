document.addEventListener("DOMContentLoaded", () => {
    const tocSidebar = document.querySelector(".toc-sidebar");
    const links = document.querySelectorAll(".toc-link");
    const listItems = document.querySelectorAll(".toc-list li");
    if (!links.length || !tocSidebar) return;

    const toggleBtn = document.createElement("button");
    toggleBtn.className = "toc-mobile-toggle";
    toggleBtn.setAttribute("aria-label", "Toggle Table of Contents");


    listItems.forEach((li) => {
        const line = document.createElement("span");
        line.className = "toc-mini-line";


        if (li.classList.contains("toc-item-h3")) {
            line.classList.add("level-mid");
        } else if (li.classList.contains("toc-item-h4")) {
            line.classList.add("level-low");
        } else {
            line.classList.add("level-top");
        }
        toggleBtn.appendChild(line);
    });

    document.body.appendChild(toggleBtn);
    const miniLines = document.querySelectorAll(".toc-mini-line");

    toggleBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        tocSidebar.classList.toggle("show-mobile");
    });


    document.addEventListener("click", (e) => {
        if (!tocSidebar.contains(e.target) && !toggleBtn.contains(e.target)) {
            tocSidebar.classList.remove("show-mobile");
        }

    });


    links.forEach(link => {
        link.addEventListener("click", () => {
            if (window.innerWidth <= 1024) tocSidebar.classList.remove("show-mobile");
        });
    });


    const headings = Array.from(links).map((link) =>
        document.getElementById(link.getAttribute("href").slice(1))
    ).filter(Boolean);

    const scrollOffset = 80;
    function updateActiveTOC() {
        if (!headings.length) return;
        let activeIndex = 0;
        for (let i = 0; i < headings.length; i++) {
            if (headings[i].getBoundingClientRect().top <= scrollOffset) {
                activeIndex = i;
            } else break;
        }

        const isAtBottom = window.innerHeight + window.scrollY >= document.body.offsetHeight - 10;
        if (isAtBottom) activeIndex = headings.length - 1;
        // Clear previous active states
        document.querySelectorAll(".toc-list li").forEach((li) => li.classList.remove("active"));
        links.forEach((link) => link.classList.remove("active"));
        miniLines.forEach((line) => line.classList.remove("active"));
        // Set new active states for both the main sidebar and the mini-map lines
        const activeLink = document.querySelector(`.toc-link[href="#${headings[activeIndex].id}"]`);
        if (activeLink) {
            activeLink.classList.add("active");
            activeLink.parentElement.classList.add("active");
            if (miniLines[activeIndex]) miniLines[activeIndex].classList.add("active");
        }
    }

    let ticking = false;
    window.addEventListener("scroll", () => {
        if (!ticking) {
            window.requestAnimationFrame(() => {
                updateActiveTOC();
                ticking = false;
            });
            ticking = true;
        }
    });
    updateActiveTOC();
});
