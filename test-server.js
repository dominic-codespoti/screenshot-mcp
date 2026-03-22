const { spawn } = require("child_process");

const child = spawn("screenshot-mcp");

child.stdout.on("data", (data) => console.log(`STDOUT: ${data.toString()}`));
child.stderr.on("data", (data) => console.log(`STDERR: ${data.toString()}`));
child.on("close", (code) => console.log(`Process exited with code ${code}`));

setInterval(() => {
   // Keep alive
}, 1000);

const initMessage = {"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{"protocolVersion":"2024-11-05","capabilities":{}, "clientInfo":{"name":"test","version":"1"}}};
child.stdin.write(JSON.stringify(initMessage) + "\n");
