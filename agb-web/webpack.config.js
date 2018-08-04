const path = require('path');

module.exports = {
	context: path.resolve(__dirname, "scripts"),
	entry: "./index.js",
	output: {
		path: path.resolve(__dirname, "out"),
		filename: "index.js",
	},
	mode: "development"
};
