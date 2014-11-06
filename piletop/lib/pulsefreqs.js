var pulseaudio = require('pulseaudio')();
var through = require('through2');
var ndarray = require('ndarray');
var fft = require('ndarray-fft');
var mag = require('ndarray-complex').mag;

var CHUNKSIZE = 4 * 2400;
var BPE = new Float32Array().BYTES_PER_ELEMENT;
var RATE = 48000;

module.exports = function(cb) {    
    pulseaudio.on('connection', function(){
        var stream = pulseaudio.record({
            stream: "my-awesome-stream",
            format: "F32LE",
            rate: RATE,
            channels: 1
        });
        var output = stream
                .pipe(chunkyStream(CHUNKSIZE))
                .pipe(through.obj(function(chunk, enc, cb) {
                    console.log("chunk", chunk.length);
                    var floats = new Float32Array(Math.floor(chunk.length / BPE));
                    for (var i = 0; i < floats.length; i++) {
                        floats[i] = chunk.readFloatLE(i * BPE);
                    }
                    var freqs = findFrequencies(floats, { rate: RATE, range: [16, 8000] });
                    this.push(freqs);
                    cb();
                }));
        cb(null, output);
    });
};

function chunkyStream(chunkSize) {
    var buf;
    return through(function(chunk, enc, cb) {
        if (buf) {
            buf = Buffer.concat([buf, chunk]);
        } else {
            buf = chunk;
        }

        while(buf.length >= chunkSize) {
            this.push(buf.slice(0, chunkSize));
            buf = buf.slice(chunkSize);
        }
        cb();
    });
}

// From https://github.com/substack/sillyscope/blob/master/index.js#L26
function findFrequencies (floats, opts) {
    var reals = ndarray(floats, [ floats.length, 1 ]);
    var imags = ndarray(zeroes(floats.length), [ floats.length, 1 ]);
    
    fft(1, reals, imags);
    mag(reals, reals, imags);
    
    var freqs = [];
    for (var i = 0; i < reals.data.length; i++) {
        var freq = i * opts.rate / floats.length;
        if (freq >= opts.range[0] && freq <= opts.range[1]) {
            freqs.push([ freq, reals.data[i] ]);
        }
    }
    return freqs;
}

function zeroes(len) {
    var arr = new Array(len);
    for(var i = 0; i < len; i++) {
        arr[i] = 0;
    }
    return arr;
}
