import { spawn } from "node:child_process";

function run(command, args, options = {}) {
  return spawn(command, args, {
    stdio: "inherit",
    shell: process.platform === "win32",
    ...options,
  });
}

function waitFor(child) {
  return new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${child.spawnargs.join(" ")} exited with ${signal ?? code}`));
      }
    });
  });
}

const leptosArgs = process.argv.slice(2);

if (leptosArgs[0] === "--") {
  leptosArgs.shift();
}

await waitFor(run("npm", ["run", "build:core-js"]));

const coreJsWatch = run("npm", ["run", "watch:core-js"]);
const leptosWatch = run("cargo", ["leptos", "watch", ...leptosArgs]);

let shuttingDown = false;

function stop(signal = "SIGTERM") {
  if (shuttingDown) {
    return;
  }

  shuttingDown = true;
  coreJsWatch.kill(signal);
  leptosWatch.kill(signal);
}

process.on("SIGINT", () => stop("SIGINT"));
process.on("SIGTERM", () => stop("SIGTERM"));

const exitCode = await new Promise((resolve) => {
  leptosWatch.on("exit", (code, signal) => {
    stop(signal ?? "SIGTERM");
    resolve(code ?? (signal ? 1 : 0));
  });
});

process.exit(exitCode);
