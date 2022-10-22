const napi = require('./napi');

napi.hello = 'from index';
console.log('hello from index');

module.exports = napi;
