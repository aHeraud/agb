# agb-web

Run agb in a browser using web-assembly.

To build, the following must be installed and on your path:

1. cargo
2. wasm-bindgen-cli
3. webpack
4. webpack-cli

And you need to install the wasm32-unknown-unknown target using the command `rustup target add wasm32-unknown-unknown`.

Then simply run build.bat/build.sh.

To run, you need to host the files in agb-web/out using a webserver that will serve the wasm file with the correct content type, and then navigate to the agb_web.html page.

To serve the page with the nodjs serve application, for example, run

```
npm install -g serve
serve ./out
```
