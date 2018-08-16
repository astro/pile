var pulsefreqs = require("../pulsefreqs");
var through = require('through2');

var bins;
var maxIntensity = 1;
var maxFreq = 8000;
pulsefreqs(function(err, src) {
    console.log("pulse src");
    maxIntensity *= 0.8;
    src.pipe(through.obj(function(freqs, enc, cb) {
        bins = binify(freqs, maxFreq, 160);
        for(var i = 0; i < bins.length; i++) {
            var intensity = bins[i];
            if (intensity > maxIntensity) {
                maxIntensity = intensity;
                console.log("maxIntensity", maxIntensity);
            }
        }
        cb();
    }));
});

module.exports = function(presets) {
    return {
        render: function(output, cb) {
            var frame = [];

            var sorted = bins && bins.sort();
            var intensity95 = sorted &&
                    sorted[Math.floor(0.95 * sorted.length)];
            intensity95 = intensity95 ? intensity95[1] : maxIntensity;
            
            for(var x = 0; x < output.x; x++) {
                var freq1 = maxFreq * x / output.x;
                var freq2 = maxFreq * (x + 1) / output.x;
                var intensity = bins && bins[x] || 0;
                if (intensity >= 0.1 * maxIntensity) {
                    // console.log({i:intensity, mi: maxIntensity, ratio: intensity / maxIntensity, log: Math.log(intensity / maxIntensity), log1: Math.log(1 + intensity / maxIntensity) });
                }
                var val = Math.max(0, Math.log(intensity) / Math.log(maxIntensity));
                var hue = Math.max(0, 360 - 360 * Math.log(1 + intensity / maxIntensity));

                var rgb = HSVtoRGB(hue, 1, val);
                var color = [rgb.r, rgb.g, rgb.b];
                if (intensity >= intensity95) {
                    // color = [255, 255, 255];
                }
                for(var y = 0; y < output.y; y++) {
                    frame.push(color);
                }
            }

            cb(null, frame);
        }
    };
};

function binify(freqs, maxFreq, count) {
    var bins = [];
    for(var x = 0; x < count; x++) {
        var freq1 = maxFreq * x / count;
        var freq2 = maxFreq * (x + 1) / count;
        var intensity = 0;
        for(var i = 0; freqs && i < freqs.length; i++) {
            var freq = freqs[i][0];
            if (freq >= freq1 && freq < freq2) {
                intensity += freqs[i][1];
            }
        }
        bins.push(intensity);
    }
    return bins;
}

function HSVtoRGB(h, s, v) {
    // this may look a bit nerdy, but everything is just
    // according to http://en.wikipedia.org/wiki/HSL_and_HSV#From_HSV
    var c = v * s;
    var h_ = (h % 360) / 60;
    var x = c * (1 - Math.abs((h_ % 2) - 1));
    var rgb = [
        [c, x, 0],
        [x, c, 0],
        [0, c, x],
        [0, x, c],
        [x, 0, c],
        [c, 0, x]
    ][Math.floor(h_)]
    // catch undefined
    rgb = rgb ? rgb : [0, 0, 0];
    var m = v - c;
    return {
        'r': Math.round((rgb[0] + m) * 255),
            'g': Math.round((rgb[1] + m) * 255),
            'b': Math.round((rgb[2] + m) * 255)
    };
}
