init();

async function init() {
    if (typeof process == "object") {
        const [{Chart}, {main, setup}] = await Promise.all([
            import("integral_site"),
            import("./index.js"),
        ]);
        setup(Chart);
        main();
    } else {
        const [{Chart, default: init}, {main, setup}] = await Promise.all([
            import("../pkg/integral_site.js"),
            import("./index.js"),
        ]);
        await init();
        setup(Chart);
        main();
    }
}
