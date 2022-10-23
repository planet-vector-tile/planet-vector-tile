// Retrieve Job-defined env vars
const { CLOUD_RUN_TASK_INDEX = 0, CLOUD_RUN_TASK_ATTEMPT = 0 } = process.env;
// Retrieve User-defined env vars
const { HELLO, WORLD } = process.env;
const { platform, arch } = process;

const pvt = require('./index');

async function main() {
    console.log(`Starting Task #${CLOUD_RUN_TASK_INDEX}, Attempt #${CLOUD_RUN_TASK_ATTEMPT}...`);

    console.log('platform', platform);
    console.log('arch', arch);

    let planet = pvt.loadPlanet('info_tile', 0, 14);
    let res = await planet.asyncMultiTwo(11);
    console.log('11 * 11', res);

    console.log(`Completed Task #${CLOUD_RUN_TASK_INDEX}.`);
}

// Start script
main().catch(err => {
    console.error(err);
    process.exit(1); // Retry Job Task by exiting the process
});
