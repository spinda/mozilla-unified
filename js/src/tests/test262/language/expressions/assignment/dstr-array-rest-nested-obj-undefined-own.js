// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-nested-obj-undefined-own.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When DestructuringAssignmentTarget is an array literal and the iterable emits `undefined` as the only value, an array with a single `undefined` element should be used as the value of the nested DestructuringAssignment. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
es6id: 13.3.2.4
features: [destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var x = null;
// Use the the top-level lexical scope for 'length' to provide compatibility with browsers
// where length and name are properties of WindowProxy
let length;

var result;
var vals = [undefined];

result = [...{ 0: x, length }] = vals;

assert.sameValue(x, undefined);
assert.sameValue(length, 1);

assert.sameValue(result, vals);

reportCompare(0, 0);
