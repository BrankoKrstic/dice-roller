import init, {
  worker_run_simulation,
} from "/pkg/dice-roller.js";

const wasmReady = init("/pkg/dice-roller.wasm");

function toI64BigInt(value) {
  const normalized = Math.trunc(Number(value));
  if (!Number.isFinite(normalized)) {
    throw new Error(`Invalid integer value: ${String(value)}`);
  }
  return BigInt(normalized);
}

self.onmessage = async (event) => {
  let request;
  try {
    request = JSON.parse(String(event.data));
  } catch (error) {
    console.log(error)

    postMessage(
      JSON.stringify({
        type: 'rro',
        content: `Invalid request payload: ${error}`,
      }),
    );
    return;
  }

  await wasmReady;

  try {
    console.time("time")
    let rawResponse;
      rawResponse = worker_run_simulation(
        request.to_hit_expression,
        request.damage_expression,
        toI64BigInt(request.target),
        Number(request.trials),
		    request.ac_mode
      );
    postMessage(rawResponse),
    console.timeEnd("time")

  } catch (error) {
    console.log(error)
    postMessage(
      JSON.stringify({ type: "Error", content: `An error occurred: ${error}`}),
    );
  }
}