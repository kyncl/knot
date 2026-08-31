document.addEventListener('DOMContentLoaded', () => {
    if (document.documentElement.getAttribute("data-theme") === "rainbow") {
        automaticChange();
    }
})


// If you hate automatic changing, replace in the Event listener `automaticChange` to `changeColor`
async function automaticChange() {
    while (true) {
        changeColor();
        await sleep(5000);
    }
};

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function changeColor() {
    const themes = [
        "crimson",
        "orange-red",
        "orange",
        "yellow",
        "green",
        "emerald",
        "teal",
        "cyan",
        "blue",
        "indigo",
        "purple",
        "rose",
        // "silver", // Doesn't look good with all the color changing
        "fuchsia",
        "pink",
        "amber",
        "lime",
        "red",
        "sky",
        "violet"
    ];
    const current = document.documentElement.getAttribute('data-theme');
    const index = Math.floor(Math.random() * themes.length);
    if (current !== themes[index]) {
        document.documentElement.setAttribute('data-theme', themes[index]);
    }
    else {
        changeColor();
    }
}
