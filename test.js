const { exec } = require('child_process');
exec('cargo run', (error, stdout, stderr) => {
    console.log(stdout, stderr);
});
