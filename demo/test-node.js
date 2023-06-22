const http = require("http");

function readBodyString(req) {
  return new Promise((resolve, reject) => {
    let body = "";
    req.on("data", (chunk) => (body += chunk));
    req.on("end", () => resolve(body));
    req.on("error", reject);
  });
}

async function hello(name) {
  if (name === "Laur") {
    throw new Error("Laur is not allowed");
  }

  return { message: `Hello, ${name}!`, thing: "A" };
}

async function handle(svc, body) {
  const data = JSON.parse(body);
  if (svc === "crate::greeting::hello") {
    const result = await hello(...data);
    return JSON.stringify({ result });
  }

  throw new Error(`Unknown service: ${svc}`);
}

const server = http.createServer(async (req, res) => {
  if (req.method !== "POST") {
    res.writeHead(405, { "Content-Type": "text/plain" });
    res.end("Method not allowed\n");
    return;
  }

  const service = req.headers["x-machinery-service"];
  if (!service) {
    res.writeHead(400, { "Content-Type": "text/plain" });
    res.end("Missing x-machinery-service header\n");
    return;
  }

  const body = await readBodyString(req);
  try {
    const result = await handle(service, body);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(result);
    return;
  } catch {
    res.writeHead(500, { "Content-Type": "text/plain" });
    res.end("Error\n");
    return;
  }
});

server.listen(9797, () => {
  console.log("Server is running...");
});
