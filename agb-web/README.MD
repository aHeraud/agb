# agb-web

Run agb in a browser using web-assembly.

## Setup:

1. You will need cargo and npm installed. Cargo can be installed from rustup.sh.

2. Install wasm-pack by running `cargo install wasm-pack`.

3. Run `npm install` from the www folder.

## Building:

1. Compile the rust library by runnning `wasm-pack build --release`.

2. Execute the command `npm link` inside the pkg folder created by the previous step.

3. Run `npm run build` in the www folder.

## Running

1. Execute `npm run start` in the www folder.

2. Webpack will start a dev server and will print the address in the console for you to open in a browser.