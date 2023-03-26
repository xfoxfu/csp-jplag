import Bottleneck from "bottleneck";
const numCPUs = require("num-cpus");
import { join as pathJoin, extname, relative } from "path";
import { promisify } from "util";
import { exec, spawn } from "child_process";
import { v4 as uuid } from "uuid";
import { stat, readdir, readFile, writeFile, readFileSync, mkdir } from "fs";
import { compile as hbs } from "handlebars";
import ProgressBar from "cli-progress";

const template = hbs(readFileSync("template.hbs").toString(), {
  //   noEscape: true,
});

console.log("Running with " + numCPUs + " concurrent jobs.");
console.log("Compile time limit " + (process.argv[2] ?? "10") + " seconds.");

const execAsync = promisify(exec);
const statAsync = promisify(stat);
const readdirAsync = promisify(readdir);
const readFileAsync = promisify(readFile);
const writeFileAsync = promisify(writeFile);
const mkdirAsync = promisify(mkdir);

// const sem = Semaphore(2 * (numCPUs ?? 8) + 1);
const sem = new Bottleneck({
  maxConcurrent: 2 * (numCPUs ?? 8) + 1,
});

const sourceDir = pathJoin(process.cwd(), "assets");
const tempDir = pathJoin(process.cwd(), "tmp");
// const runguard = pathJoin(process.cwd(), "runguard");

const results: Record<string, [boolean, string, string, number]> = {};

const checkCompile = async (
  source: string,
  type: string
): Promise<[boolean, string, number]> => {
  const filename = uuid() + type;
  const path = pathJoin(tempDir, filename);
  await writeFileAsync(path, source);

  const [code, message] = await sem.schedule(
    () =>
      new Promise((ok, err) => {
        let message = "";
        const ls = spawn("g++", [
          //   "-u",
          //   "nobody",
          //   "-g",
          //   "nobody",
          //   "-T",
          //   process.argv[2] ?? "10",
          //   "-m",
          //   "262144",
          //   "--",

          path,
          "-o",
          "/dev/null",
          "-Wall",
          "-Wextra",
          "-std=c++14",
        ]);

        ls.stdout.on("data", (data) => {
          message += data;
        });

        ls.stderr.on("data", (data) => {
          message += data;
        });

        ls.on("close", (code) => {
          ok([code, message]);
        });
      })
  );

  return [code === 0, message, code];
};

const discover = async (dir: string): Promise<string[]> => {
  const result = [];
  for (const entry of await readdirAsync(dir)) {
    const entryPath = pathJoin(dir, entry);
    const fstat = await statAsync(entryPath);
    if (fstat.isDirectory()) {
      result.push(...(await discover(entryPath)));
    } else if (fstat.isFile()) {
      result.push(entryPath);
    }
  }
  return result;
};

const checkSources = async (
  sources: string[],
  progress?: ProgressBar.SingleBar
) => {
  await Promise.all(
    sources.map(async (source) => {
      const content = (await readFileAsync(source)).toString();
      const [ok, log, code] = await checkCompile(content, extname(source));
      console.log(source, ok ? "PASS" : "FAIL");
      results[source] = [ok, log, content, code];
      progress?.increment();
    })
  );
};

const main = async () => {
  try {
    await mkdirAsync(tempDir, {});
  } catch (e) {}
  const sources = await discover(sourceDir);
  const progress = new ProgressBar.SingleBar({});
  progress.start(sources.length, 0);
  await checkSources(sources, progress);
  progress.stop();
  await writeFileAsync(
    "report.html",
    template({
      results: Object.entries(results).map(
        ([path, [ok, message, source, code]]) => ({
          path: relative(sourceDir, path),
          ok,
          message,
          source,
          code,
        })
      ),
    })
  );
  console.log("report written to report.html");
};

main().catch(console.error);
