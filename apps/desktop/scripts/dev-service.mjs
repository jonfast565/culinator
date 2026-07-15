import process from "node:process";
import { URL } from "node:url";
import { spawn } from "node:child_process";
import readline from "node:readline";

const service = spawn(
  "cargo",
  [
    "run",
    "-p",
    "culinograph-service",
    "--bin",
    "culinograph-service",
    "--",
    "--db",
    "../../target/culinograph-dev.sqlite3",
    "--origin",
    "http://localhost:1420",
  ],
  { cwd: new URL("../../../", import.meta.url), stdio: ["inherit", "pipe", "inherit"] },
);

const lines = readline.createInterface({ input: service.stdout });
let vite;
lines.on("line", (line) => {
  if (vite) {
    process.stdout.write(`${line}\n`);
    return;
  }
  try {
    const bootstrap = JSON.parse(line);
    if (!bootstrap.endpoint || !bootstrap.websocketUrl || !bootstrap.token)
      throw new Error("invalid bootstrap");
    vite = spawn("npm", ["run", "dev"], {
      cwd: new URL("../", import.meta.url),
      stdio: "inherit",
      env: {
        ...process.env,
        VITE_CULINOGRAPH_API_URL: bootstrap.endpoint,
        VITE_CULINOGRAPH_WS_URL: bootstrap.websocketUrl,
        VITE_CULINOGRAPH_API_TOKEN: bootstrap.token,
      },
    });
    vite.on("exit", () => service.kill("SIGTERM"));
  } catch {
    process.stdout.write(`${line}\n`);
  }
});

const shutdown = () => {
  vite?.kill("SIGTERM");
  service.kill("SIGTERM");
};
process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);
service.on("exit", (code) => {
  if (!vite) process.exit(code ?? 1);
});
