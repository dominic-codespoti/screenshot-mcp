const https = require('https');

function search(query) {
    return new Promise((resolve, reject) => {
        const req = https.get('https://html.duckduckgo.com/html/?q=' + encodeURIComponent(query), (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                const results = [];
                const regex = /<a class="result__snippet[^>]*>(.*?)<\/a>/gs;
                let match;
                while ((match = regex.exec(data)) !== null) {
                    results.push(match[1].replace(/<\/?[^>]+(>|$)/g, ""));
                }
                resolve(results);
            });
        });
        req.on('error', reject);
    });
}

(async () => {
    console.log("=== Query 1 ===");
    console.log(await search("github vscode copilot MCP image_content attachment multimodal"));
    console.log("=== Query 2 ===");
    console.log(await search("model context protocol vscode chat image sas url support"));
})();
