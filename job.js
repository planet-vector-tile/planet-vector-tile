// Retrieve Job-defined env vars
const {CLOUD_RUN_TASK_INDEX = 0, CLOUD_RUN_TASK_ATTEMPT = 0} = process.env;
// Retrieve User-defined env vars
const {SLEEP_MS, FAIL_RATE} = process.env;

// Define main script
const main = async () => {
  console.log(
    `Starting Task #${CLOUD_RUN_TASK_INDEX}, Attempt #${CLOUD_RUN_TASK_ATTEMPT}...`
  );
  // Simulate work
  if (SLEEP_MS) {
    await sleep(SLEEP_MS);
  }
  // Simulate errors
  if (FAIL_RATE) {
    try {
      randomFailure(FAIL_RATE);
    } catch (err) {
      err.message = `Task #${CLOUD_RUN_TASK_INDEX}, Attempt #${CLOUD_RUN_TASK_ATTEMPT} failed.\n\n${err.message}`;
      throw err;
    }
  }
  console.log(`Completed Task #${CLOUD_RUN_TASK_INDEX}.`);
};

// Wait for a specific amount of time
const sleep = ms => {
  return new Promise(resolve => setTimeout(resolve, ms));
};

// Throw an error based on fail rate
const randomFailure = rate => {
  rate = parseFloat(rate);
  if (!rate || rate < 0 || rate > 1) {
    console.warn(
      `Invalid FAIL_RATE env var value: ${rate}. Must be a float between 0 and 1 inclusive.`
    );
    return;
  }

  const randomFailure = Math.random();
  if (randomFailure < rate) {
    throw new Error('Task failed.');
  }
};

// Start script
main().catch(err => {
  console.error(err);
  process.exit(1); // Retry Job Task by exiting the process
});