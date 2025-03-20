const babelOptions = {
  presets: ["babel-preset-gatsby", "@babel/preset-typescript", "@babel/preset-react"],
}

module.exports = require("babel-jest").default.createTransformer(babelOptions)