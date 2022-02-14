init();

async function init() {
    if (typeof process == "object") {
        const [{ChartManager}, {main, setup}] = await Promise.all([
            import("integral_site"),
            import("./index.js"),
        ]);
        setup(ChartManager);
        main();
    } else {
        const [{ChartManager, default: init}, {main, setup}] = await Promise.all([
            import("../pkg/integral_site.js"),
            import("./index.js"),
        ]);
        await init();
        setup(ChartManager);
        main();
    }
}
