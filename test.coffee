PEG = require 'pegjs'
fs = require 'fs'
util = require 'util'

grammar = fs.readFileSync './rat.pegjs', {encoding:"utf8"}
source = fs.readFileSync './temps.rat', {encoding:"utf8"}

p = PEG.buildParser grammar
x = p.parse(source)
console.log(JSON.stringify(x, null, 2))
