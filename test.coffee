PEG = require 'pegjs'
fs = require 'fs'
util = require 'util'
_ = require 'underscore'

grammar = fs.readFileSync './rat.pegjs', {encoding:"utf8"}
source = fs.readFileSync './temps.rat', {encoding:"utf8"}

p = PEG.buildParser grammar
x = _.compact p.parse source
console.log(JSON.stringify(x, null, 2))
