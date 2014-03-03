PEG = require 'pegjs'
fs = require 'fs'
util = require 'util'
_ = require 'underscore'
Springy = require 'springy'

grammar = fs.readFileSync './rat.pegjs', {encoding:"utf8"}
source = fs.readFileSync './temps.rat', {encoding:"utf8"}

g = new Springy.Graph()
p = PEG.buildParser grammar
v = _.compact p.parse source

nodes = []

_.flatten(v[0]["lines"]).map((y) -> 
	label = if (y.args) == undefined then y.proc else y.proc + "(" + y.args + ")"
	nodes.push [y.proc+y.pos.x+y.pos.y, {"label": label}]
)

b = v[0]["lines"]

edges = []

b.forEach((e, i, l) ->
	e.forEach (ee, ii, ll) ->
		if i != 0
			if ee.dyad != "^"
				upper = l[i-1][ii]
				upperName = upper.proc + upper.pos.x + upper.pos.y
			upperName = edges[edges.length-1][0] if ee.dyad == "^"
			localName = ee.proc + ee.pos.x + ee.pos.y
			edges.push [upperName, localName]
			if ee.dyad == "v"
				upper = l[i+1][ii-1]
				upperName = upper.proc + upper.pos.x + upper.pos.y
				edges.push [localName, upperName]
		)

console.log(JSON.stringify({"edges": edges, "nodes": nodes}, null, 2))
